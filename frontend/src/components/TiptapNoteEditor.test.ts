import { describe, expect, it } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TiptapNoteEditor from './TiptapNoteEditor.vue'

const emptyDoc = { type: 'doc', content: [] }

describe('TiptapNoteEditor', () => {
  it('mounts with empty content', () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    expect(wrapper.find('.tiptap-wrapper').exists()).toBe(true)
  })

  it('shows toolbar with math insert buttons', () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    const toolbar = wrapper.find('.tiptap-toolbar')
    expect(toolbar.exists()).toBe(true)
    expect(toolbar.text()).toContain('$f$')
    expect(toolbar.text()).toContain('$$f$$')
  })

  it('hides toolbar when editable is false', () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: false },
    })
    expect(wrapper.find('.tiptap-toolbar').exists()).toBe(false)
  })

  it('exposes editor instance', () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    const vm = wrapper.vm as unknown as { editor: unknown }
    expect(vm.editor).toBeDefined()
  })

  it('toolbar hint mentions click-to-edit and enter-to-math', () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    expect(wrapper.find('.toolbar-hint').text()).toContain('Enter')
    expect(wrapper.find('.toolbar-hint').text()).toContain('点击')
  })

  it('emits update when toolbar button inserts formula', async () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    await wrapper.find('.toolbar-btn[title="插入行内公式"]').trigger('click')
    await flushPromises()
    expect(wrapper.emitted('update')).toBeTruthy()
  })

  it('can set content with inline math and retrieve JSON', async () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    await flushPromises()

    const vm = wrapper.vm as unknown as {
      editor: { getJSON: () => Record<string, unknown>; commands: { setContent: (c: unknown) => void } }
    }

    vm.editor.commands.setContent({
      type: 'doc',
      content: [{
        type: 'paragraph',
        content: [
          { type: 'text', text: 'Formula: ' },
          { type: 'inlineMath', attrs: { latex: 'E=mc^2' } },
        ],
      }],
    })
    await flushPromises()

    const json = vm.editor.getJSON()
    const content = json.content as Array<Record<string, unknown>>
    const para = content[0] as Record<string, unknown>
    const paraContent = para.content as Array<Record<string, unknown>>
    const mathNode = paraContent.find((n) => n.type === 'inlineMath')
    expect(mathNode).toBeDefined()
    expect((mathNode?.attrs as Record<string, string>)?.latex).toBe('E=mc^2')
  })

  it('can set content with block math and retrieve JSON', async () => {
    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })
    await flushPromises()

    const vm = wrapper.vm as unknown as {
      editor: { getJSON: () => Record<string, unknown>; commands: { setContent: (c: unknown) => void } }
    }

    vm.editor.commands.setContent({
      type: 'doc',
      content: [
        { type: 'blockMath', attrs: { latex: '\\int_0^1 x dx = \\frac{1}{2}' } },
      ],
    })
    await flushPromises()

    const json = vm.editor.getJSON()
    const content = json.content as Array<Record<string, unknown>>
    const mathNode = content.find((n) => n.type === 'blockMath')
    expect(mathNode).toBeDefined()
    expect((mathNode?.attrs as Record<string, string>)?.latex).toBe('\\int_0^1 x dx = \\frac{1}{2}')
  })

  it('updates when content prop changes externally', async () => {
    const docWithText = {
      type: 'doc',
      content: [{ type: 'paragraph', content: [{ type: 'text', text: 'hello' }] }],
    }

    const wrapper = mount(TiptapNoteEditor, {
      props: { content: emptyDoc, editable: true },
    })

    await wrapper.setProps({ content: docWithText })
    await flushPromises()

    const vm = wrapper.vm as unknown as { editor: { getJSON: () => Record<string, unknown> } }
    const json = vm.editor.getJSON()
    const content = json.content as Array<Record<string, unknown>>
    expect(content).toHaveLength(1)
    expect(content[0].type).toBe('paragraph')
  })
})
