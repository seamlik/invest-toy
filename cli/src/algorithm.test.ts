import { describe, expect, test } from '@jest/globals'
import { scoreAndRankGeneric, scoreAndRankGenericInverted } from './algorithm'

describe('scoreAndRankGeneric', () => {
  const candidates = new Map<string, number>([
    ['A', 1],
    ['B', 2],
    ['C', 3],
    ['D', 4]
  ])

  test('rank candidates', () => {
    expect(scoreAndRankGeneric(candidates)).toEqual(new Map<string, number>(
      [
        ['A', 0.1],
        ['B', 0.2],
        ['C', 0.3],
        ['D', 0.4]
      ]
    ))
  })

  test('rank candidates in revert', () => {
    expect(scoreAndRankGenericInverted(candidates)).toEqual(new Map<string, number>(
      [
        ['A', 0.4],
        ['B', 0.3],
        ['C', 0.2],
        ['D', 0.1]
      ]
    ))
  })
})
