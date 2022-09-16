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
    const PERatioList = await this.fetchPERatio(portfolio.map(position => position.conid))
    const PERatioMapByTicker = new Map<string, number>()
    portfolio.forEach((position, index) => {
      const PERatio = PERatioList[index]
      if (PERatio !== undefined) {
        PERatioMapByTicker.set(position.ticker, PERatio)
      }
    })

    const factors = new Map<string, ScoringFactors>()

    for (const position of portfolio) {
      const marketHistory = await this.fetchHistoricalMarketData(position.conid)
      const earlistEntry = marketHistory[0]
      const ticker = position.ticker
      const PERatio = PERatioMapByTicker.get(ticker)
      if (earlistEntry === undefined) {
        console.warn(`${ticker} has no market history`)
        factors.set(ticker, new ScoringFactors(PERatio, undefined))
        progressBar.increment()
      } else {
        const fromPrice = earlistEntry.c
        const latestEntry = marketHistory[marketHistory.length - 1]
        if (latestEntry === undefined) {
          console.warn(`${ticker} has only 1 entry in market history`)
          factors.set(ticker, new ScoringFactors(PERatio, undefined))
          progressBar.increment()
        } else {
          const toPrice = latestEntry.c
          if (fromPrice === 0) {
            console.warn(`${ticker} has a 0 as its price a month ago`)
            factors.set(ticker, new ScoringFactors(PERatio, undefined))
            progressBar.increment()
          } else {
            const change = (toPrice - fromPrice) / fromPrice
            factors.set(ticker, new ScoringFactors(PERatio, change))
            progressBar.increment()
          }
        }
      }
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
    const endpoint = `iserver/marketdata/history?conid=${conid}&period=1m&bar=1d&outsideRth=true`
    return (await fetchIbkr(endpoint) as HistoricalMarketData)
      .data
      .sort((a, b) => a.t - b.t)
  }

  async fetchPERatio (conids: number[]): Promise<Array<number | undefined>> {
    const conidsText = conids.join(',')
    const endpoint = `iserver/marketdata/snapshot?conids=${conidsText}&fields=${FIELD_ID_PE_RATIO}`
    return (await fetchIbkr(endpoint) as unknown[]).map(parsePERatio)
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
