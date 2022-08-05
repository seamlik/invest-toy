import { env } from 'node:process'

class Toy {
  accountId = ''

  async run (): Promise<void> {
    // Account ID
    const accounts = await fetchIbkr('portfolio/accounts') as Account[]
    const defaultAccount = accounts[0]
    if (defaultAccount === undefined) {
      throw new Error('No account found')
    } else {
      this.accountId = defaultAccount.accountId
    }

    // Portfolio
    const portfolio = await this.fetchPortfolio()
    console.info(`Found ${portfolio.length} stocks`)

    // Finalize report
    const report: ReportEntry[] = []
    for (const position of portfolio) {
      const data = await this.fetchHistoricalMarketData(position.conid)
      const earlistEntry = data[0]
      const ticker = position.ticker
      if (earlistEntry === undefined) {
        report.push({ ticker: ticker })
      } else {
        const fromDate = renderDate(earlistEntry.t)
        const fromPrice = earlistEntry.c
        const latestEntry = data[data.length - 1]
        if (latestEntry === undefined) {
          report.push({ ticker, fromDate, fromPrice })
        } else {
          const toPrice = latestEntry.c
          const entry: ReportEntry = {
            ticker,
            fromDate,
            fromPrice,
            toDate: renderDate(latestEntry.t),
            toPrice

          }
          if (fromPrice !== 0) {
            const changeRaw = (toPrice - fromPrice) / fromPrice
            entry.changeRaw = changeRaw
            entry.change = `${(changeRaw * 100).toFixed(2)}%`
          }
          report.push(entry)
        }
      }
    }
    report.sort(sortRecord)

    // Write the report
    const columns = [
      'ticker',
      'change',
      'fromDate',
      'fromPrice',
      'toDate',
      'toPrice'
    ]
    console.table(report, columns)
  }

  async fetchPortfolioAtPage (pageIndex: number): Promise<PortfolioPosition[]> {
    return await fetchIbkr(`portfolio/${this.accountId}/positions/${pageIndex}`) as PortfolioPosition[]
  }

  async fetchPortfolio (): Promise<PortfolioPosition[]> {
    // Fetch the first page always
    let currentPageIndex = 0
    const positions: PortfolioPosition[] = await this.fetchPortfolioAtPage(0)
    let currentPageSize = positions.length

    while (currentPageSize >= 30) {
      const nextPage = await this.fetchPortfolioAtPage(++currentPageIndex)
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
}

async function fetchIbkr (endpoint: string): Promise<unknown> {
  const headers = new Headers()
  headers.set('User-Agent', 'Rust')

  const endpointFull = `https://127.0.0.1:5000/v1/api/${endpoint}`
  const response = await fetch(endpointFull, { headers: headers })
  if (!response.ok) {
    throw new Error(`${response.statusText}: ${await response.text()}`)
  }

  return await response.json()
}

function renderDate (timestamp: number): string {
  return new Date(timestamp).toLocaleDateString()
}

interface Account {
  accountId: string
}

interface PortfolioPosition {
  conid: number
  ticker: string
}

interface HistoricalMarketData {
  data: HistoricalMarketDataEntry[]
}

interface HistoricalMarketDataEntry {
  /**
   * Price at market close
   */
  c: number

  /**
   * Timestamp
   */
  t: number
}

interface ReportEntry {
  ticker: string
  fromDate?: string
  fromPrice?: number
  toDate?: string
  toPrice?: number

  change?: string
  changeRaw?: number
}

function sortRecord (a: ReportEntry, b: ReportEntry): number {
  if (a.changeRaw !== undefined && b.changeRaw !== undefined) {
    return a.changeRaw - b.changeRaw
  } else if (a.changeRaw === undefined && b.changeRaw !== undefined) {
    return 1
  } else if (a.changeRaw !== undefined && b.changeRaw === undefined) {
    return -1
  } else {
    return 0
  }
}

async function main (): Promise<void> {
  // IBKR Gateway uses a self-signed TLS certificate
  env['NODE_TLS_REJECT_UNAUTHORIZED'] = '0'
  return await new Toy().run()
}

await main()

// Dummy export to make this file an ES module, so that we can use top-level await.
export {}
