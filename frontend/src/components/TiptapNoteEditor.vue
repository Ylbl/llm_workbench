<script setup lang="ts">
import { onBeforeUnmount, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Mathematics from '@tiptap/extension-mathematics'
import Placeholder from '@tiptap/extension-placeholder'
import { TextSelection } from '@tiptap/pm/state'
import 'katex/dist/katex.min.css'

const props = defineProps<{
  content: Record<string, unknown>
  editable?: boolean
}>()

const emit = defineEmits<{
  update: [json: Record<string, unknown>]
  save: []
}>()

function insertInlineFormula() {
  if (!editor.value) return
  editor.value.chain().focus().insertContent({
    type: 'inlineMath',
    attrs: { latex: 'x=0' },
  }).run()
}

function insertBlockFormula() {
  if (!editor.value) return
  editor.value.chain().focus().insertContent({
    type: 'blockMath',
    attrs: { latex: 'x = 0' },
  }).run()
}

function handleMathEnter(): boolean {
  if (!editor.value) return false

  const { state } = editor.value
  const { from } = state.selection
  const $from = state.doc.resolve(from)

  const parent = $from.parent
  if (!parent.isTextblock) return false

  const parentStart = $from.start()
  const text = parent.textContent

  if (text.endsWith('$$') && text.length > 2) {
    const openIdx = text.indexOf('$$')
    if (openIdx >= 0 && openIdx < text.length - 2) {
      const latex = text.slice(openIdx + 2, -2).trim()
      const tr = state.tr
      tr.delete(parentStart + openIdx, parentStart + text.length)
      const nodeType = state.schema.nodes.blockMath
      if (nodeType) {
        tr.insert(parentStart + openIdx, nodeType.create({ latex }))
        editor.value.view.dispatch(tr)
      }
      return true
    }
  }

  const localOffset = from - parentStart - 1
  if (
    localOffset >= 0 &&
    localOffset < text.length &&
    text[localOffset] === '$'
  ) {
    const beforeDollar = text.slice(0, localOffset)
    const openIdx = beforeDollar.lastIndexOf('$')
    if (openIdx >= 0) {
      const latex = text.slice(openIdx + 1, localOffset).trim()
      if (latex.length === 0) return false

      const afterDollar = text.slice(localOffset + 1)
      if (afterDollar.trim().length > 0) return false

      const tr = state.tr
      tr.delete(parentStart + openIdx, parentStart + text.length)
      const nodeType = state.schema.nodes.inlineMath
      if (nodeType) {
        tr.insert(parentStart + openIdx, nodeType.create({ latex }))
        tr.insertText(' ', parentStart + openIdx + 1)
        editor.value.view.dispatch(tr)
      }
      return true
    }
  }

  return false
}

const editor = useEditor({
  content: props.content,
  editable: props.editable ?? true,
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Mathematics.configure({
      katexOptions: {
        throwOnError: false,
        errorColor: '#cc0000',
      },
    }),
    Placeholder.configure({
      placeholder: '开始写笔记...',
    }),
  ],
  onUpdate: () => {
    if (!editor.value) return
    emit('update', editor.value.getJSON() as Record<string, unknown>)
  },
  editorProps: {
    attributes: {
      class: 'tiptap-editor',
    },
    handleKeyDown: (_view, event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 's') {
        event.preventDefault()
        emit('save')
        return true
      }
      if (event.key === 'Enter' && !event.shiftKey) {
        if (handleMathEnter()) return true
      }
      return false
    },
    handleClickOn(view, _pos, node, nodePos, _event) {
      if (node.type.name === 'inlineMath') {
        const latex = node.attrs.latex as string
        const tr = view.state.tr
        const nodeSize = node.nodeSize
        tr.delete(nodePos, nodePos + nodeSize)
        const text = `$${latex}$`
        tr.insertText(text, nodePos)
        tr.setSelection(TextSelection.create(tr.doc, nodePos + 1))
        view.dispatch(tr)
        return true
      }
      if (node.type.name === 'blockMath') {
        const latex = node.attrs.latex as string
        const tr = view.state.tr
        const nodeSize = node.nodeSize
        tr.delete(nodePos, nodePos + nodeSize)
        const text = `$$\n${latex}\n$$`
        tr.insertText(text, nodePos)
        tr.setSelection(TextSelection.create(tr.doc, nodePos + text.length - 2))
        view.dispatch(tr)
        return true
      }
      return false
    },
  },
})

watch(
  () => props.content,
  (newContent) => {
    if (!editor.value) return
    const currentJson = JSON.stringify(editor.value.getJSON())
    const newJson = JSON.stringify(newContent)
    if (currentJson !== newJson) {
      editor.value.commands.setContent(newContent)
    }
  },
)

watch(
  () => props.editable,
  (editable) => {
    editor.value?.setEditable(editable ?? true)
  },
)

onBeforeUnmount(() => {
  editor.value?.destroy()
})

defineExpose({ editor })
</script>

<template>
  <div class="tiptap-wrapper">
    <div v-if="editable !== false" class="tiptap-toolbar">
      <button
        class="toolbar-btn"
        type="button"
        title="插入行内公式"
        @click="insertInlineFormula()"
      >
        $f$
      </button>
      <button
        class="toolbar-btn"
        type="button"
        title="插入块公式"
        @click="insertBlockFormula()"
      >
        $$f$$
      </button>
      <span class="toolbar-hint">
        $...$ + Enter → inline math, $$...$$ + Enter → block, 点击公式可编辑
      </span>
    </div>
    <EditorContent :editor="editor" />
  </div>
</template>

<style>
.tiptap-wrapper {
  background: var(--surface, #fff);
  border-radius: 6px;
}

.tiptap-toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-bottom: 1px solid var(--border, #d9e1e8);
  background: var(--surface-subtle, #f1f5f7);
}

.toolbar-btn {
  padding: 4px 10px;
  border: 1px solid var(--border, #d9e1e8);
  border-radius: 4px;
  background: var(--surface, #fff);
  color: var(--text, #1e2933);
  font-family: var(--font-mono, monospace);
  font-size: 13px;
  cursor: pointer;
}

.toolbar-btn:hover {
  background: var(--surface-subtle, #f1f5f7);
  border-color: var(--accent, #0f766e);
  color: var(--accent, #0f766e);
}

.toolbar-hint {
  margin-left: 8px;
  font-size: 11px;
  color: var(--muted, #64707d);
}

.tiptap-editor {
  padding: 16px 20px;
  min-height: 320px;
  outline: none;
  font-size: 15px;
  line-height: 1.7;
}

.tiptap-editor p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  color: var(--muted, #64707d);
  float: left;
  height: 0;
  pointer-events: none;
  font-style: italic;
}

.tiptap-editor h1 {
  font-size: 28px;
  line-height: 1.3;
  margin: 0 0 8px;
}

.tiptap-editor h2 {
  font-size: 22px;
  line-height: 1.3;
  margin: 0 0 6px;
}

.tiptap-editor h3 {
  font-size: 18px;
  line-height: 1.3;
  margin: 0 0 4px;
}

.tiptap-editor p {
  margin: 0 0 8px;
}

.tiptap-editor ul,
.tiptap-editor ol {
  padding-left: 24px;
  margin: 0 0 8px;
}

.tiptap-editor li {
  margin-bottom: 4px;
}

.tiptap-editor blockquote {
  border-left: 3px solid var(--border, #d9e1e8);
  padding-left: 16px;
  margin: 0 0 8px;
  color: var(--muted, #64707d);
}

.tiptap-editor pre {
  background: var(--surface-subtle, #f1f5f7);
  border-radius: 6px;
  padding: 12px 16px;
  margin: 0 0 8px;
  font-family: var(--font-mono);
  font-size: 14px;
  overflow-x: auto;
}

.tiptap-editor code {
  background: var(--surface-subtle, #f1f5f7);
  padding: 2px 4px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 0.9em;
}

.tiptap-editor pre code {
  background: none;
  padding: 0;
}

.tiptap-editor .katex-display {
  margin: 12px 0;
}

.tiptap-editor .katex {
  font-size: 1.1em;
}

.tiptap-editor .katex-display > .katex {
  font-size: 1.21em;
}

.tiptap-editor .math-node {
  cursor: pointer;
  border-radius: 3px;
  padding: 0 2px;
}

.tiptap-editor .math-node:hover {
  background: rgba(15, 118, 110, 0.08);
  outline: 1px solid rgba(15, 118, 110, 0.2);
}
</style>
