export type SettingsMap = Record<string, unknown>

interface SettingsResponse {
  settings: SettingsMap
}

export async function fetchSettings(): Promise<SettingsMap> {
  const response = await fetch('/api/settings', {
    headers: {
      Accept: 'application/json',
    },
  })

  if (!response.ok) {
    throw new Error(`Settings load failed with HTTP ${response.status}`)
  }

  const payload = (await response.json()) as SettingsResponse
  return payload.settings
}

export async function patchSettings(settings: SettingsMap): Promise<SettingsMap> {
  const response = await fetch('/api/settings', {
    method: 'PATCH',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ settings }),
  })

  if (!response.ok) {
    throw new Error(`Settings save failed with HTTP ${response.status}`)
  }

  const payload = (await response.json()) as SettingsResponse
  return payload.settings
}
