const BASE = '/api'

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message)
    this.name = 'ApiError'
  }
}

export async function api<T>(path: string, init?: RequestInit & { params?: Record<string, string | number | undefined> }): Promise<T> {
  const url = new URL(`${BASE}${path}`, window.location.origin)
  if (init?.params) {
    Object.entries(init.params).forEach(([key, value]) => {
      if (value !== undefined) url.searchParams.set(key, String(value))
    })
  }
  const res = await fetch(url.toString(), {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new ApiError(res.status, body.error || 'Request failed')
  }
  return res.json()
}

export function fetchSSE(
  url: string,
  onChunk: (text: string) => void,
  onDone: () => void,
  onError: (err: Error) => void,
) {
  const evtSource = new EventSource(`${BASE}${url}`)
  evtSource.onmessage = (e) => {
    if (e.data === '[DONE]') {
      evtSource.close()
      onDone()
      return
    }
    onChunk(e.data)
  }
  evtSource.onerror = () => {
    evtSource.close()
    onError(new Error('SSE connection error'))
  }
  return () => evtSource.close()
}
