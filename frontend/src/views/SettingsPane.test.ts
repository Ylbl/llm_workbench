import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import SettingsPane from './SettingsPane.vue'

describe('SettingsPane', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('loads settings into the JSON editor', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        json: async () => ({
          settings: {
            editor: {
              fontSize: 16,
            },
          },
        }),
      })),
    )

    const wrapper = mount(SettingsPane, {
      global: {
        plugins: [createPinia()],
      },
    })

    await flushPromises()

    const editor = wrapper.get('textarea')
    expect((editor.element as HTMLTextAreaElement).value).toContain('"fontSize": 16')
    expect(fetch).toHaveBeenCalledWith('/api/settings', {
      headers: {
        Accept: 'application/json',
      },
    })
  })
})
