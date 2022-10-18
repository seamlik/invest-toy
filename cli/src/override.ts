import { ScoringFactors } from './algorithm.js'

/**
 * Manually overrides the scores of some stocks because market data of some exchanges are not
 * available.
 */
export const SCORE_OVERRIDES = new Map(
  [
    ['NESN', new ScoringFactors(18.55, 0.2717, 0.009)]
  ]
)
