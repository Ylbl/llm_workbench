export interface HealthResponse {
  service: string
  version: string
  status: string
  app: {
    host: string
    port: number
    app_data_dir: string
  }
  database: {
    configured: boolean
    status: string
  }
}

export async function fetchHealth(): Promise<HealthResponse> {
  const response = await fetch('/api/health', {
    headers: {
      Accept: 'application/json',
    },
  })

  if (!response.ok) {
    throw new Error(`Health check failed with HTTP ${response.status}`)
  }

  return response.json() as Promise<HealthResponse>
}
