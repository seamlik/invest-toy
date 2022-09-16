// Data models in the REST API from IBKR

export interface PortfolioAccount {
  accountId: string
}

export interface IserverAccount {
  accounts: string[]
}

export interface PortfolioPosition {
  conid: number
  ticker: string
  position: number
  assetClass: string
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
