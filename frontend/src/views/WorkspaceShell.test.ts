import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import WorkspaceShell from './WorkspaceShell.vue'

vi.mock('vue-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-router')>()
  return {
    ...actual,
    useRoute: () => ({ name: 'workspace' }),
  }
})

describe('WorkspaceShell', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('renders the app shell and loaded health status', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
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
      })),
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

    expect(wrapper.text()).toContain('LLM Workbench')
    expect(wrapper.text()).toContain('System Status')
    expect(wrapper.text()).toContain('ok')
    expect(wrapper.text()).toContain('not_configured')
    expect(fetch).toHaveBeenCalledWith('/api/health', {
      headers: {
        Accept: 'application/json',
      },
    })
  })
})
