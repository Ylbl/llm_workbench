import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import MixedContentRenderer from './MixedContentRenderer.vue'

describe('MixedContentRenderer', () => {
  it('renders plain text as-is', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: 'Hello world' },
    })
    expect(wrapper.text()).toContain('Hello world')
  })

  it('renders inline math alongside text', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: 'The formula $E=mc^2$ is famous' },
    })
    expect(wrapper.text()).toContain('The formula')
    expect(wrapper.text()).toContain('is famous')
    expect(wrapper.html()).toContain('katex')
  })

  it('renders block math', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: 'Before\n\n$$x^2$$\n\nAfter' },
    })
    expect(wrapper.html()).toContain('katex')
    expect(wrapper.text()).toContain('Before')
    expect(wrapper.text()).toContain('After')
  })

  it('leaves incomplete formula as raw text', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: 'Formula $E = mc' },
    })
    expect(wrapper.html()).not.toContain('katex')
    expect(wrapper.text()).toContain('$E = mc')
  })

  it('handles mixed inline and block in order', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: '$a=1$ and\n\n$$b=2$$\n\nend' },
    })
    const html = wrapper.html()
    const inlinePos = html.indexOf('katex')
    const secondKatexPos = html.indexOf('katex', inlinePos + 5)
    expect(inlinePos).toBeGreaterThan(0)
    expect(secondKatexPos).toBeGreaterThan(0)
    expect(wrapper.text()).toContain('and')
    expect(wrapper.text()).toContain('end')
  })

  it('handles empty input', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: '' },
    })
    expect(wrapper.html()).not.toContain('katex')
  })

  it('handles invalid LaTeX without crashing', () => {
    const wrapper = mount(MixedContentRenderer, {
      props: { content: 'Bad formula: $\\invalid{command$ ' },
    })
    expect(wrapper.html()).toBeDefined()
  })
})
