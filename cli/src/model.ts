// Data models in the REST API from IBKR

export interface Account {
  accountId: string
}

export interface PortfolioPosition {
  conid: number
  ticker: string
}

export interface HistoricalMarketData {
  data: HistoricalMarketDataEntry[]
}

export interface HistoricalMarketDataEntry {
  /**
     * Price at market close
     */
  c: number

  /**
     * Timestamp
     */
  t: number
}
