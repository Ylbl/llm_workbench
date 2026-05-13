import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import MathRenderer from './MathRenderer.vue'

describe('MathRenderer', () => {
  it('renders inline formula as span', () => {
    const wrapper = mount(MathRenderer, {
      props: { latex: 'E=mc^2', displayMode: false },
    })
    expect(wrapper.find('span.math-inline').exists()).toBe(true)
    expect(wrapper.html()).toContain('katex')
  })

  it('renders block formula as div', () => {
    const wrapper = mount(MathRenderer, {
      props: { latex: 'x^2 + y^2 = 1', displayMode: true },
    })
    expect(wrapper.find('div.math-block').exists()).toBe(true)
    expect(wrapper.html()).toContain('katex')
  })

  it('falls back gracefully for invalid LaTeX', () => {
    const wrapper = mount(MathRenderer, {
      props: { latex: '\\invalid{command', displayMode: false },
    })
    expect(wrapper.find('.math-renderer').exists()).toBe(true)
    expect(wrapper.html()).toContain('katex-error')
  })

  it('uses displayMode false by default', () => {
    const wrapper = mount(MathRenderer, {
      props: { latex: 'x=1' },
    })
    expect(wrapper.find('span.math-inline').exists()).toBe(true)
  })
})
