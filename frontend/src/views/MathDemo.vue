<script setup lang="ts">
import { ref } from 'vue'
import MathRenderer from '../components/MathRenderer.vue'
import MixedContentRenderer from '../components/MixedContentRenderer.vue'

const customLatex = ref('')

const examples = [
  {
    label: 'Inline $...$',
    text: 'Einstein says $E = mc^2$ in his famous equation.',
  },
  {
    label: 'Inline \\(...\\)',
    text: 'The quadratic formula \\(x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\\) is useful.',
  },
  {
    label: 'Block $$...$$',
    text: 'A famous identity:\n\n$$\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}$$\n\nThis is the Gaussian integral.',
  },
  {
    label: 'Block \\[...\\]',
    text: 'Definition:\\n\\n\\[\\nabla \\cdot \\mathbf{E} = \\frac{\\rho}{\\varepsilon_0}\\]\\nThis is Gauss\'s law.',
  },
  {
    label: 'Mixed inline',
    text: 'Values: $\\alpha = 0.5$, $\\beta = \\sqrt{2}$, $\\gamma = \\frac{1}{3}$. Done.',
  },
  {
    label: 'Incomplete (streaming)',
    text: 'The equation $E = mc',
  },
  {
    label: 'Invalid LaTeX',
    text: 'Here is an invalid formula: $\\invalid{command$ should fail gracefully.',
  },
  {
    label: 'Normal text only',
    text: 'This is normal text without any formulas.',
  },
  {
    label: 'Multiple blocks',
    text: 'First:\n\n$$a = b + c$$\n\nSecond:\n\n$$d = e \\cdot f$$',
  },
]

const standaloneFormulas = [
  { latex: 'x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}', displayMode: false, label: 'Inline' },
  { latex: '\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}', displayMode: true, label: 'Block' },
  { latex: '\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}', displayMode: true, label: 'Matrix' },
  { latex: '\\invalid{command', displayMode: false, label: 'Invalid fallback' },
]
</script>

<template>
  <section class="math-demo">
    <div class="section-title-row">
      <div>
        <h2>Math Rendering Demo</h2>
        <p class="section-subtitle">Batch 3 — Shared Math Layer</p>
      </div>
    </div>

    <div class="demo-section">
      <h3>Standalone Formulas</h3>
      <div class="demo-grid">
        <div v-for="formula in standaloneFormulas" :key="formula.label" class="demo-card">
          <div class="demo-card-label">{{ formula.label }}</div>
          <div class="demo-card-code">\{{ formula.latex }}</div>
          <div class="demo-card-render">
            <MathRenderer :latex="formula.latex" :displayMode="formula.displayMode" />
          </div>
        </div>
      </div>
    </div>

    <div class="demo-section">
      <h3>Mixed Content Rendering</h3>
      <div v-for="example in examples" :key="example.label" class="demo-example">
        <div class="demo-example-header">
          <span class="demo-example-label">{{ example.label }}</span>
        </div>
        <div class="demo-example-input">
          <pre class="demo-example-raw">{{ example.text }}</pre>
        </div>
        <div class="demo-example-output">
          <MixedContentRenderer :content="example.text" />
        </div>
      </div>
    </div>

    <div class="demo-section">
      <h3>Try Your Own</h3>
      <textarea
        v-model="customLatex"
        class="demo-input"
        placeholder="Enter text with $formulas$ or $$display math$$"
        rows="4"
      />
      <div v-if="customLatex" class="demo-example-output" style="margin-top: 12px">
        <MixedContentRenderer :content="customLatex" />
      </div>
    </div>
  </section>
</template>

<style scoped>
.math-demo {
  margin: 24px 32px 0;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-subtitle {
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 14px;
}

.demo-section {
  margin-bottom: 28px;
}

.demo-section h3 {
  margin: 0 0 12px;
  font-size: 15px;
  color: var(--muted);
}

.demo-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.demo-card {
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-subtle);
}

.demo-card-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--muted);
  text-transform: uppercase;
  margin-bottom: 6px;
}

.demo-card-code {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--muted);
  margin-bottom: 10px;
  padding: 6px 8px;
  background: var(--bg);
  border-radius: 4px;
  overflow-x: auto;
}

.demo-card-render {
  padding: 10px 0;
}

.demo-example {
  margin-bottom: 20px;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.demo-example-header {
  padding: 8px 12px;
  background: var(--surface-subtle);
  border-bottom: 1px solid var(--border);
}

.demo-example-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
}

.demo-example-input {
  padding: 10px 12px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}

.demo-example-raw {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--muted);
  white-space: pre-wrap;
  word-break: break-word;
}

.demo-example-output {
  padding: 14px 16px;
  background: var(--surface);
}

.demo-input {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-subtle);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: 14px;
  resize: vertical;
}

.demo-input:focus {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}

@media (max-width: 860px) {
  .math-demo {
    margin-inline: 16px;
  }

  .demo-grid {
    grid-template-columns: 1fr;
  }
}
</style>
