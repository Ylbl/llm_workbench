import katex from 'katex'

export interface KatexRenderOptions {
  throwOnError: boolean
  errorColor: string
  strict: boolean | 'error' | 'ignore' | 'warn'
  trust: boolean
  displayMode: boolean
  output: 'html' | 'mathml' | 'htmlAndMathml'
}

export const defaultRenderOptions: KatexRenderOptions = {
  throwOnError: false,
  errorColor: '#cc0000',
  strict: false,
  trust: false,
  displayMode: false,
  output: 'html',
}

export const inlineMathRenderOptions: KatexRenderOptions = {
  ...defaultRenderOptions,
  displayMode: false,
}

export const blockMathRenderOptions: KatexRenderOptions = {
  ...defaultRenderOptions,
  displayMode: true,
}

export type MathSegment =
  | { type: 'text'; content: string }
  | { type: 'inline-math'; latex: string }
  | { type: 'block-math'; latex: string }

interface DelimiterMatch {
  open: string
  close: string
  block: boolean
}

function tokenizeMath(source: string): MathSegment[] {
  const segments: MathSegment[] = []
  let pos = 0

  while (pos < source.length) {
    const match = findNextDelimiter(source, pos)

    if (!match) {
      segments.push({ type: 'text', content: source.slice(pos) })
      break
    }

    const closePos = findClosingDelimiter(source, match.start + match.open.length, match.close)

    if (closePos === -1) {
      segments.push({ type: 'text', content: source.slice(pos) })
      break
    }

    if (match.start > pos) {
      segments.push({ type: 'text', content: source.slice(pos, match.start) })
    }

    const latex = source.slice(match.start + match.open.length, closePos).trim()
    segments.push({
      type: match.block ? 'block-math' : 'inline-math',
      latex,
    })

    pos = closePos + match.close.length
  }

  return segments
}

function findNextDelimiter(source: string, from: number): (DelimiterMatch & { start: number }) | null {
  for (let i = from; i < source.length; i++) {
    if (source[i] === '\\') {
      const rest = source.slice(i)
      if (rest.startsWith('\\[')) return { start: i, open: '\\[', close: '\\]', block: true }
      if (rest.startsWith('\\(')) return { start: i, open: '\\(', close: '\\)', block: false }
      i++ // skip escaped char
      continue
    }

    if (source[i] === '$') {
      if (source[i + 1] === '$') {
        return { start: i, open: '$$', close: '$$', block: true }
      }
      // Single $ for inline: verify it's not preceded by a letter (avoid currency confusion)
      if (i === 0 || !/[a-zA-Z0-9]/.test(source[i - 1])) {
        return { start: i, open: '$', close: '$', block: false }
      }
    }
  }

  return null
}

function findClosingDelimiter(source: string, from: number, closer: string): number {
  for (let i = from; i < source.length; i++) {
    if (source.slice(i, i + closer.length) === closer) {
      // For inline $, don't match if followed by a letter/number (currency)
      if (closer === '$' && i + 1 < source.length && /[a-zA-Z0-9]/.test(source[i + 1])) {
        continue
      }
      return i
    }
  }
  return -1
}

export function parseMixedContent(text: string): MathSegment[] {
  if (!text) return []
  return tokenizeMath(text)
}

export function renderFormula(latex: string, displayMode: boolean): string {
  try {
    return katex.renderToString(latex, {
      ...defaultRenderOptions,
      displayMode,
    })
  } catch {
    return `<span class="math-fallback">${escapeHtml(latex)}</span>`
  }
}

export function renderFormulaOrFallback(latex: string, displayMode: boolean): string {
  return renderFormula(latex, displayMode)
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}
