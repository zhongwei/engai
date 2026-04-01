const BASE = '/api';

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || 'Request failed');
  }
  return res.json();
}

export function fetchSSE(
  url: string,
  onChunk: (text: string) => void,
  onDone: () => void,
  onError: (err: Error) => void,
) {
  const evtSource = new EventSource(`${BASE}${url}`);
  evtSource.onmessage = (e) => {
    if (e.data === '[DONE]') {
      evtSource.close();
      onDone();
      return;
    }
    onChunk(e.data);
  };
  evtSource.onerror = () => {
    evtSource.close();
    onError(new Error('SSE connection error'));
  };
  return () => evtSource.close();
}
