import { describe, expect, it } from 'vitest'
import { parseMixedContent, renderFormula, type MathSegment } from '../utils/math'

describe('parseMixedContent', () => {
  it('returns empty array for empty input', () => {
    expect(parseMixedContent('')).toEqual([])
  })

  it('treats plain text as single text segment', () => {
    const result = parseMixedContent('Hello world')
    expect(result).toEqual([{ type: 'text', content: 'Hello world' }])
  })

  it('detects inline math with $...$', () => {
    const result = parseMixedContent('The formula $E=mc^2$ is famous')
    expect(result).toHaveLength(3)
    expect(result[0]).toEqual({ type: 'text', content: 'The formula ' })
    expect(result[1]).toEqual({ type: 'inline-math', latex: 'E=mc^2' })
    expect(result[2]).toEqual({ type: 'text', content: ' is famous' })
  })

  it('detects inline math with \\(...\\)', () => {
    const result = parseMixedContent('Value \\(x=5\\) is set')
    expect(result).toHaveLength(3)
    expect(result[1]).toEqual({ type: 'inline-math', latex: 'x=5' })
  })

  it('detects block math with $$...$$', () => {
    const result = parseMixedContent('Before\n\n$$x^2+y^2=1$$\n\nAfter')
    expect(result).toHaveLength(3)
    expect(result[1]).toEqual({ type: 'block-math', latex: 'x^2+y^2=1' })
  })

  it('detects block math with \\[...\\]', () => {
    const result = parseMixedContent('Eq:\n\\[\\int_0^1 x dx\\]\nDone')
    expect(result).toHaveLength(3)
    expect(result[1]).toEqual({ type: 'block-math', latex: '\\int_0^1 x dx' })
  })

  it('leaves incomplete inline formula as raw text', () => {
    const result = parseMixedContent('The formula $E=mc is incomplete')
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'text', content: 'The formula $E=mc is incomplete' })
  })

  it('leaves incomplete block formula as raw text', () => {
    const result = parseMixedContent('Start\n$$incomplete block\nend')
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'text', content: 'Start\n$$incomplete block\nend' })
  })

  it('handles multiple inline formulas in one text', () => {
    const result = parseMixedContent('$a=1$ and $b=2$')
    expect(result).toHaveLength(3)
    expect(result[0]).toEqual({ type: 'inline-math', latex: 'a=1' })
    expect(result[1]).toEqual({ type: 'text', content: ' and ' })
    expect(result[2]).toEqual({ type: 'inline-math', latex: 'b=2' })
  })

  it('handles mixed inline and block formulas', () => {
    const result = parseMixedContent('Inline $x=1$ and block\n\n$$y=2$$\n\ndone')
    expect(result).toHaveLength(5)
    expect(result[0]).toEqual({ type: 'text', content: 'Inline ' })
    expect(result[1]).toEqual({ type: 'inline-math', latex: 'x=1' })
    expect(result[2]).toEqual({ type: 'text', content: ' and block\n\n' })
    expect(result[3]).toEqual({ type: 'block-math', latex: 'y=2' })
    expect(result[4]).toEqual({ type: 'text', content: '\n\ndone' })
  })

  it('distinguishes $$ from $ (block vs inline)', () => {
    const result = parseMixedContent('$$block$$\n$inline$')
    expect(result).toHaveLength(3)
    expect(result[0]).toEqual({ type: 'block-math', latex: 'block' })
    expect(result[1]).toEqual({ type: 'text', content: '\n' })
    expect(result[2]).toEqual({ type: 'inline-math', latex: 'inline' })
  })

  it('treats streaming-like text ending with $ as raw', () => {
    const result = parseMixedContent('Current: $E = mc')
    expect(result).toHaveLength(1)
    expect(result[0].type).toBe('text')
    expect(result[0].content).toContain('$E = mc')
  })

  it('treats streaming-like text ending with $$ as raw', () => {
    const result = parseMixedContent('Block: $$E = mc')
    expect(result).toHaveLength(1)
    expect(result[0].type).toBe('text')
  })
})

describe('renderFormula', () => {
  it('renders valid inline formula', () => {
    const html = renderFormula('E=mc^2', false)
    expect(html).toContain('katex')
    expect(html).toContain('E')
  })

  it('renders valid block formula', () => {
    const html = renderFormula('\\sum_{n=1}^{\\infty} \\frac{1}{n^2}', true)
    expect(html).toContain('katex')
  })

  it('returns error HTML for invalid LaTeX without throwing', () => {
    const html = renderFormula('\\invalid{command', false)
    expect(html).toBeDefined()
    // KaTeX with throwOnError: false renders error span, doesn't throw
    expect(html).toContain('katex-error')
    expect(html).toContain('\\invalid{command')
  })

  it('returns safe fallback for empty formula', () => {
    const html = renderFormula('', false)
    expect(html).toBeDefined()
  })
})
