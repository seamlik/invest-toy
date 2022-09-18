// Main logic

import { SingleBar } from 'cli-progress'
import {
  RenderedReportEntry,
  scoreAndRank,
  ScoringFactors
} from './algorithm.js'
import {
  HistoricalMarketData,
  HistoricalMarketDataEntry,
  IserverAccount,
  PortfolioAccount,
  PortfolioPosition
} from './model.js'
import { lastEntryOf } from './util.js'

const FIELD_ID_PE_RATIO = 7290
const LONG_TERM_YEARS = 8

export class Toy {
  accountId = ''

  async run (): Promise<void> {
    // Account ID
    const portfolioAccounts = await fetchIbkr('portfolio/accounts') as PortfolioAccount[]

    // Some API requires querying this endpoint first
    const iserverAccounts = await fetchIbkr('iserver/accounts') as IserverAccount
    if (iserverAccounts.accounts.length === 0) {
      throw new Error('No brokerage account found')
    }

    const defaultAccount = portfolioAccounts[0]
    if (defaultAccount === undefined) {
      throw new Error('No default account found')
    } else {
      this.accountId = defaultAccount.accountId
    }

    // Portfolio
    const portfolio = await this.fetchPortfolio()
    console.info(`Found ${portfolio.length} stocks`)

    // Progress bar
    const progressBar = new SingleBar({})
    progressBar.start(portfolio.length, 0)

    try {
      // Finalize report
      const report = await this.fetchReport(portfolio, progressBar)
      progressBar.stop()

      // Write the report
      console.table(report)
    } finally {
      progressBar.stop()
    }
  }

  async fetchReport (
    portfolio: PortfolioPosition[],
    progressBar: SingleBar
  ): Promise<RenderedReportEntry[]> {
    const PERatioMapByTicker = await this.fetchPERatio(portfolio)
    const factors = new Map<string, ScoringFactors>()

    for (const position of portfolio) {
      const marketHistorySinceLastMonth = await this.fetchHistoricalMarketData(
        position.conid,
        MarketHistoryPeriod.SHORT_TERM
      )
      const marketHistorySinceLongTerm = await this.fetchHistoricalMarketData(
        position.conid,
        MarketHistoryPeriod.LONG_TERM
      )
      const earliestEntry = lastEntryOf(marketHistorySinceLongTerm)
      const latestEntry = marketHistorySinceLastMonth[0]
      const lastMonthEntry = this.lastMonthEntryOf(marketHistorySinceLastMonth)
      const ticker = position.ticker
      const PERatio = PERatioMapByTicker.get(ticker)
      if (latestEntry === undefined) {
        console.warn(`${ticker} has no market history`)
        factors.set(ticker, new ScoringFactors(PERatio))
      } else {
        let longTermChange: number | undefined
        if (marketHistorySinceLongTerm.length < LONG_TERM_YEARS * 12 - 4) {
          console.warn(`${ticker} has not enough long-term history (${marketHistorySinceLongTerm.length} months), ignoring the long-term change.`)
        } else {
          longTermChange = this.calculateChange(
            ticker,
            MarketHistoryPeriod.LONG_TERM,
            latestEntry,
            earliestEntry
          )
        }

        let shortTermChange: number | undefined
        if (marketHistorySinceLastMonth.length < 30) {
          console.warn(`${ticker} has not enough short-term history (${marketHistorySinceLastMonth.length} months), ignoring the short-term change.`)
        } else {
          shortTermChange = this.calculateChange(
            ticker,
            MarketHistoryPeriod.SHORT_TERM,
            latestEntry,
            lastMonthEntry
          )
        }

        factors.set(ticker, new ScoringFactors(PERatio, longTermChange, shortTermChange))
      }

      progressBar.increment()
    }

    return scoreAndRank(factors)
  }

  lastMonthEntryOf (history: HistoricalMarketDataEntry[]): HistoricalMarketDataEntry | undefined {
    const latest = history[0]
    if (latest === undefined) {
      return undefined
    }

    const millisecondsOf1Month = 1000 * 60 * 60 * 24 * 30
    for (const entry of history) {
      if (latest.t - entry.t >= millisecondsOf1Month) {
        return entry
      }
    }

    return undefined
  }

  calculateChange (
    ticker: string,
    period: MarketHistoryPeriod,
    latestEntry: HistoricalMarketDataEntry,
    earliestEntry?: HistoricalMarketDataEntry
  ): number | undefined {
    if (earliestEntry === undefined) {
      console.warn(`${ticker} has no earliest price, cannot calculate the ${MarketHistoryPeriod[period]} change.`)
      return undefined
    } else if (earliestEntry.c === 0) {
      console.warn(`${ticker} has a 0 as its ealiest price, cannot calculate the ${MarketHistoryPeriod[period]} change.`)
      return undefined
    } else {
      return (latestEntry.c - earliestEntry.c) / earliestEntry.c
    }
  }

  async fetchPortfolioAtPage (pageIndex: number): Promise<PortfolioPosition[]> {
    const allPositions = await fetchIbkr(
      `portfolio/${this.accountId}/positions/${pageIndex}`
    ) as PortfolioPosition[]
    return allPositions.filter(pos => pos.assetClass === 'STK')
  }

  async fetchPortfolio (): Promise<PortfolioPosition[]> {
    // Fetch the first page always
    // Filter out entries with 0 position because IBKR still include stocks I recently sold
    let currentPageIndex = 0
    const positions = (await this.fetchPortfolioAtPage(0)).filter(entry => entry.position !== 0)
    let currentPageSize = positions.length

    while (currentPageSize >= 30) {
      const nextPage = (await this.fetchPortfolioAtPage(++currentPageIndex)).filter(entry => entry.position !== 0)
      currentPageSize = nextPage.length
      positions.push(...nextPage)
    }

    return positions
  }

  /**
     * Fetches historical market data of a stock of this IBKR contract ID.
     *
     * The entries are sorted according to their timestamp.
     */
  async fetchHistoricalMarketData (
    conid: number,
    period: MarketHistoryPeriod
  ): Promise<HistoricalMarketDataEntry[]> {
    let paramPeriod: string
    let paramBar: string
    switch (period) {
      case MarketHistoryPeriod.LONG_TERM:
        paramPeriod = `${LONG_TERM_YEARS}y`
        paramBar = '1m'
        break
      case MarketHistoryPeriod.SHORT_TERM:
        paramPeriod = '2m'
        paramBar = '1d'
        break
    }
    const endpoint = `iserver/marketdata/history?conid=${conid}&period=${paramPeriod}&bar=${paramBar}&outsideRth=false`
    return (await fetchIbkr(endpoint) as HistoricalMarketData)
      .data
      .sort((a, b) => b.t - a.t)
  }

  async fetchPERatio (portfolio: PortfolioPosition[]): Promise<Map<string, number>> {
    const conidsText = portfolio.map(pos => pos.conid).join(',')
    const endpoint = `iserver/marketdata/snapshot?conids=${conidsText}&fields=${FIELD_ID_PE_RATIO}`
    const PERatioList = (await fetchIbkr(endpoint) as unknown[]).map(parsePERatio)
    const PERatioMapByTicker = new Map<string, number>()
    portfolio.forEach((position, index) => {
      const PERatio = PERatioList[index]
      if (PERatio !== undefined) {
        PERatioMapByTicker.set(position.ticker, PERatio)
      }
    })
    return PERatioMapByTicker
  }
}

function parsePERatio (data: unknown): number | undefined {
  if (typeof (data) !== 'object') {
    return undefined
  }

  const map = new Map(Object.entries(data as object))
  const PERatio = map.get(FIELD_ID_PE_RATIO.toString())
  if (typeof (PERatio) === 'string') {
    return parseFloat(PERatio)
  } else {
    return undefined
  }
}

async function fetchIbkr (endpoint: string): Promise<unknown> {
  const headers = new Headers()
  headers.set('User-Agent', 'Rust')

  const endpointFull = `https://127.0.0.1:5000/v1/api/${endpoint}`
  const response = await fetch(endpointFull, { headers })
  if (!response.ok) {
    throw new Error(`${response.statusText}: ${await response.text()}`)
  }

  return await response.json()
}

enum MarketHistoryPeriod {
  LONG_TERM,
  SHORT_TERM,
}
