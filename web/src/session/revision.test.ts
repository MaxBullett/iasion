import { describe, expect, it } from 'vitest'
import { revisionLabel } from './revision'

describe('revisionLabel', () => {
  it('abbreviates a commit to seven characters', () => {
    expect(revisionLabel('0123456789abcdef0123456789abcdef01234567')).toBe('0123456')
  })

  it('shows a local build as it is', () => {
    expect(revisionLabel('dev')).toBe('dev')
  })

  it('leaves anything else unchanged rather than guessing', () => {
    expect(revisionLabel('0123456')).toBe('0123456')
  })
})
