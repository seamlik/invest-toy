export function scoreAndRank (candidates: Map<string, ScoringFactors>): RenderedReportEntry[] {
  const report: ReportEntry[] = []

  // Price change
  let totalChangeQuantity = 0
  for (const candidate of candidates.values()) {
    if (candidate.change !== undefined && candidate.change < 0) {
      // Choose stocks that dropped the most
      totalChangeQuantity += Math.abs(candidate.change)
    }
  }

  // Dividend yield
  let totalDividendYieldQuantity = 0
  for (const candidate of candidates.values()) {
    if (candidate.dividendYield !== undefined && candidate.dividendYield > 0) {
      // Choose stocks that dropped the most
      totalDividendYieldQuantity += candidate.dividendYield
    }
  }

  candidates.forEach((factor, ticker) => {
    const changeIsFavored =
      totalChangeQuantity === 0 ||
      factor.change === undefined ||
      factor.change >= 0
    const scoreForChange = changeIsFavored ? 0 : Math.abs(factor.change / totalChangeQuantity)

    const dividendYieldIsFavored =
      totalDividendYieldQuantity === 0 ||
      factor.dividendYield === undefined ||
      factor.dividendYield <= 0
    const scoreForDividendYield = dividendYieldIsFavored
      ? 0
      : factor.dividendYield / totalDividendYieldQuantity

    const entry = new ReportEntry(
      ticker,
      scoreForChange + scoreForDividendYield,
      factor.dividendYield,
      factor.change
    )
    report.push(entry)
  })

  report.sort(ReportEntry.sort)
  return report.map(entry => entry.render())
}

export class ScoringFactors {
  constructor (
    /**
     * Percentage point of dividend yield.
     *
     * Negative means data is unavailable or it has no dividend.
     */
    public readonly dividendYield: number,

    /**
     * Change in stock prices.
     *
     * Positive means the price increased, while negative means the price decreased.
     * Undefined means data is unavailable.
     */
    public readonly change?: number
  ) {}
}

class ReportEntry {
  constructor (
    public readonly ticker: string,
    public readonly score: number,
    public readonly dividendYield: number,
    public readonly change?: number
  ) {}

  render (): RenderedReportEntry {
    const unknownText = 'Unknown'
    return new RenderedReportEntry(
      this.ticker,
      this.score < 0 ? unknownText : (this.score * 100).toFixed(2),
      this.dividendYield <= 0 ? 'None' : `${this.dividendYield}%`,
      this.change === undefined ? unknownText : `${(this.change * 100).toFixed(2)}%`
    )
  }

  static sort (a: ReportEntry, b: ReportEntry): number {
    return b.score - a.score
  }
}

export class RenderedReportEntry {
  constructor (
    public readonly ticker: string,
    public readonly score: string,
    public readonly dividendYield: string,
    public readonly change: string
  ) {}
}
