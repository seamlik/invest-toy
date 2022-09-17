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
      const marketHistory = await this.fetchHistoricalMarketData(position.conid)
      const earliestEntry = marketHistory[0]
      const latestEntry = marketHistory[marketHistory.length - 1]
      const secondLatestEntry = marketHistory[marketHistory.length - 2]
      const ticker = position.ticker
      const PERatio = PERatioMapByTicker.get(ticker)
      if (earliestEntry === undefined) {
        console.warn(`${ticker} has no market history`)
        factors.set(ticker, new ScoringFactors(PERatio))
      } else {
        if (latestEntry === undefined) {
          console.warn(`${ticker} has only 1 entry in market history, cannot calculate any changes.`)
          factors.set(ticker, new ScoringFactors(PERatio))
        } else {
          let longTermChange: number | undefined
          if (earliestEntry.c === 0) {
            console.warn(`${ticker} has a 0 as its ealiest price, cannot calculate the long-term change.`)
          } else if (marketHistory.length < 12 * LONG_TERM_YEARS - 1) {
            console.warn(`${ticker} has a market history shorter than ${LONG_TERM_YEARS} years (${marketHistory.length} months), ignoring the long-term change.`)
          } else {
            longTermChange = (latestEntry.c - earliestEntry.c) / earliestEntry.c
          }

          let recentChange: number | undefined
          if (secondLatestEntry.c === 0) {
            console.warn(`${ticker} has a 0 as its price from the last month, cannot calculate the recent change.`)
          } else {
            recentChange = (latestEntry.c - secondLatestEntry.c) / secondLatestEntry.c
          }

          factors.set(ticker, new ScoringFactors(PERatio, longTermChange, recentChange))
        }
      }
      progressBar.increment()
    }

    return scoreAndRank(factors)
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
    const positions: PortfolioPosition[] = (await this.fetchPortfolioAtPage(0)).filter(entry => entry.position !== 0)
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
  async fetchHistoricalMarketData (conid: number): Promise<HistoricalMarketDataEntry[]> {
    const endpoint = `iserver/marketdata/history?conid=${conid}&period=${LONG_TERM_YEARS}y&bar=1m&outsideRth=true`
    return (await fetchIbkr(endpoint) as HistoricalMarketData)
      .data
      .sort((a, b) => a.t - b.t)
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
