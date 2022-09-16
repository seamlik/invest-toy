export function scoreAndRank (candidates: Map<string, ScoringFactors>): RenderedReportEntry[] {
  // Price change
  const candidatesOfRecentChange = new Map<string, number>()
  candidates.forEach((factors, ticker) => {
    if (factors.change !== undefined && factors.change < 0) {
      // Choose stocks that dropped the most
      candidatesOfRecentChange.set(ticker, Math.abs(factors.change))
    }
  })
  const scoresOfRecentChange = scoreAndRankGeneric(candidatesOfRecentChange)

  // Price over earnings
  const candidatesOfPERatio = new Map<string, number>()
  candidates.forEach((factors, ticker) => {
    if (factors.PERatio !== undefined && factors.PERatio > 0) {
      candidatesOfPERatio.set(ticker, factors.PERatio)
    }
  })
  const scoresOfPERatio = scoreAndRankGenericInverted(candidatesOfPERatio)

  console.info(scoresOfPERatio)

  // Aggregate scores
  const report: ReportEntry[] = []
  candidates.forEach((factors, ticker) => {
    report.push(new ReportEntry(
      ticker,
      (scoresOfRecentChange.get(ticker) ?? 0) + (scoresOfPERatio.get(ticker) ?? 0),
      factors.PERatio,
      factors.change
    ))
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
    public readonly PERatio?: number,
    public readonly change?: number
  ) {}

  render (): RenderedReportEntry {
    const unknownText = 'Unknown'
    return new RenderedReportEntry(
      this.ticker,
      this.score < 0 ? unknownText : (this.score * 100).toFixed(2),
      this.PERatio?.toFixed(2) ?? 'None',
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
    public readonly PERatio: string,
    public readonly change: string
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
