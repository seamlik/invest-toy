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
      (scoresOfShortTermChange.get(ticker) ?? 0) +
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
     * Average monthly change in stock prices in the long term.
     *
     * Same representation as `shortTermChange`.
     */
    public readonly longTermChange?: number,

    /**
     * 1-month change in stock prices.
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

export function scoreAndRankGeneric (candidates: Map<string, number>): Map<string, number> {
  const totalNotional = totalNotionalOf(candidates)
  const scores = new Map<string, number>()
  candidates.forEach((value, key) => scores.set(key, value / totalNotional))
  return scores
}

export function scoreAndRankGenericInverted (candidates: Map<string, number>): Map<string, number> {
  const keysSortedByValue: string[] = []
  const valuesSorted: number[] = []
  candidates.forEach((v, k) => {
    keysSortedByValue.push(k)
    valuesSorted.push(v)
  })
  keysSortedByValue.sort((a, b) => (candidates.get(a) ?? 0) - (candidates.get(b) ?? 0))
  valuesSorted.sort((a, b) => a - b)

  const invertedCandidates = new Map<string, number>()
  keysSortedByValue.forEach(
    (key, index) => invertedCandidates.set(key, valuesSorted[valuesSorted.length - index - 1])
  )
  return scoreAndRankGeneric(invertedCandidates)
}

function totalNotionalOf (candidates: Map<string, number>): number {
  let totalNotional = 0
  for (const entry of candidates.values()) {
    totalNotional += entry
  }
  return totalNotional
}
