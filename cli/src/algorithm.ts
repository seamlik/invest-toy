export function scoreAndRank (candidates: Map<string, ScoringFactors>): RenderedReportEntry[] {
  // Recent change
  const candidatesOfShortTermChange = new Map<string, number>()
  candidates.forEach((factors, ticker) => {
    if (factors.shortTermChange !== undefined && factors.shortTermChange < 0) {
      // Choose stocks that dropped the most recently
      candidatesOfShortTermChange.set(ticker, Math.abs(factors.shortTermChange))
    }
  })
  const scoresOfShortTermChange = scoreAndRankGeneric(candidatesOfShortTermChange)

  // Long-term change
  const candidatesOfLongTermChange = new Map<string, number>()
  candidates.forEach((factors, ticker) => {
    if (factors.longTermChange !== undefined && factors.longTermChange > 0) {
      // Choose stocks that increased the most in the long term
      candidatesOfLongTermChange.set(ticker, factors.longTermChange)
    }
  })
  const scoresOfLongTermChange = scoreAndRankGeneric(candidatesOfLongTermChange)

  // Price over earnings
  const candidatesOfPERatio = new Map<string, number>()
  candidates.forEach((factors, ticker) => {
    if (factors.PERatio !== undefined && factors.PERatio > 0) {
      candidatesOfPERatio.set(ticker, factors.PERatio)
    }
  })
  const scoresOfPERatio = scoreAndRankGenericInverted(candidatesOfPERatio)

  // Aggregate scores
  const report: ReportEntry[] = []
  candidates.forEach((factors, ticker) => {
    const totalScore =
      (scoresOfShortTermChange.get(ticker) ?? 0) * 2 +
      (scoresOfLongTermChange.get(ticker) ?? 0) +
      (scoresOfPERatio.get(ticker) ?? 0)
    report.push(new ReportEntry(ticker, totalScore, factors))
  })

  report.sort(ReportEntry.sort)
  return report.map(entry => entry.render())
}

export class ScoringFactors {
  constructor (
    /**
     * Price over earnings.
     */
    public readonly PERatio?: number,

    /**
     * Change of the stock price in the long term.
     *
     * Same representation as `shortTermChange`.
     */
    public readonly longTermChange?: number,

    /**
     * Change of the stock price in the short term.
     *
     * Positive means the price increased, while negative means the price decreased.
     * Undefined means data is unavailable.
     */
    public readonly shortTermChange?: number
  ) {}
}

class ReportEntry {
  private readonly unknownText = 'Unknown'

  constructor (
    public readonly ticker: string,
    public readonly score: number,
    public readonly factors: ScoringFactors
  ) {}

  render (): RenderedReportEntry {
    return new RenderedReportEntry(
      this.ticker,
      this.score < 0 ? this.unknownText : (this.score * 100).toFixed(2),
      this.factors.PERatio?.toFixed(2) ?? 'None',
      this.renderChange(this.factors.shortTermChange),
      this.renderChange(this.factors.longTermChange)
    )
  }

  private renderChange (change?: number): string {
    return change === undefined ? this.unknownText : `${(change * 100).toFixed(2)}%`
  }

  static sort (a: ReportEntry, b: ReportEntry): number {
    return b.score - a.score
  }
}

export class RenderedReportEntry {
  constructor (
    public readonly ticker: string,
    public readonly score: string,
    public readonly PERatio: string,
    public readonly shortTermChange: string,
    public readonly longTermChange: string
  ) {}
}
