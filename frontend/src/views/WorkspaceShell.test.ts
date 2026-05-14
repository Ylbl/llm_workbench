import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import WorkspaceShell from './WorkspaceShell.vue'

vi.mock('vue-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-router')>()
  return {
    ...actual,
    useRoute: () => ({ name: 'workspace', params: {} }),
    useRouter: () => ({ push: vi.fn() }),
  }
})

describe('WorkspaceShell', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    localStorage.clear()
  })

  it('renders the app shell and loaded health status', async () => {
    const workspaceItems: unknown[] = []

    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input)
        if (url.includes('/api/workspace/items')) {
          return { ok: true, status: 200, json: async () => workspaceItems }
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            service: 'llm_workbench',
            version: '0.1.0',
            status: 'ok',
            app: {
              host: '127.0.0.1',
              port: 3000,
              app_data_dir: './data',
            },
            database: {
              configured: false,
              status: 'not_configured',
            },
          }),
        }
      }),
    )

    const wrapper = mount(WorkspaceShell, {
      global: {
        plugins: [createPinia()],
        stubs: {
          RouterLink: {
            template: '<a><slot /></a>',
          },
        },
      },
    })

    await flushPromises()

    expect(wrapper.text()).toContain('llm_workbench')
    expect(wrapper.text()).toContain('项目')
    expect(wrapper.text()).toContain('大纲')
    expect(wrapper.find('.ide-topbar').exists()).toBe(false)
    expect(wrapper.find('.project-panel-actions').exists()).toBe(false)
    expect(wrapper.text()).toContain('ok')
    expect(wrapper.text()).toContain('not_configured')
    expect(fetch).toHaveBeenCalledWith('/api/health', {
      headers: {
        Accept: 'application/json',
      },
    })
  })

  it('collapses the right panel without leaving its resize column', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input)
        if (url.includes('/api/workspace/items')) {
          return { ok: true, status: 200, json: async () => [] }
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            service: 'llm_workbench',
            version: '0.1.0',
            status: 'ok',
            app: {
              host: '127.0.0.1',
              port: 3000,
              app_data_dir: './data',
            },
            database: {
              configured: false,
              status: 'not_configured',
            },
          }),
        }
      }),
    )

    const wrapper = mount(WorkspaceShell, {
      global: {
        plugins: [createPinia()],
        stubs: {
          RouterLink: {
            template: '<a><slot /></a>',
          },
        },
      },
    })

    await flushPromises()

    expect(wrapper.find('.right-tool-window').exists()).toBe(true)
    expect(wrapper.find('.panel-resizer-right').exists()).toBe(true)

    await wrapper.find('.right-tool-window .icon-button.compact').trigger('click')

    expect(wrapper.find('.right-tool-window').exists()).toBe(false)
    expect(wrapper.find('.panel-resizer-right').exists()).toBe(false)
    expect(wrapper.find('.activity-bar-right').exists()).toBe(true)
  })

  it('resizes side panels with the drag separator', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input)
        if (url.includes('/api/workspace/items')) {
          return { ok: true, status: 200, json: async () => [] }
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({
            service: 'llm_workbench',
            version: '0.1.0',
            status: 'ok',
            app: {
              host: '127.0.0.1',
              port: 3000,
              app_data_dir: './data',
            },
            database: {
              configured: false,
              status: 'not_configured',
            },
          }),
        }
      }),
    )

    const wrapper = mount(WorkspaceShell, {
      global: {
        plugins: [createPinia()],
        stubs: {
          RouterLink: {
            template: '<a><slot /></a>',
          },
        },
      },
    })

    await flushPromises()
    wrapper
      .find('.panel-resizer-left')
      .element.dispatchEvent(new MouseEvent('pointerdown', { clientX: 300, bubbles: true }))
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 350 }))
    await nextTick()

    expect(wrapper.find('.left-tool-window').attributes('style')).toContain('width: 342px')

    window.dispatchEvent(new MouseEvent('pointerup'))

    expect(localStorage.getItem('llm-workbench:left-panel-width')).toBe('342')
  })
})
