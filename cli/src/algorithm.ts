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
