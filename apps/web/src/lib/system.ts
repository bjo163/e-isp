export interface SystemInfo {
  name: string;
  version: string;
  api_version: string;
  status: string;
  runtime: string;
}

export type ApiConnection =
  | { connected: true; info: SystemInfo }
  | { connected: false; reason: 'unconfigured' | 'unavailable' };

export async function getApiConnection(): Promise<ApiConnection> {
  const baseUrl = process.env.API_INTERNAL_URL?.trim();
  if (!baseUrl) return { connected: false, reason: 'unconfigured' };

  try {
    const response = await fetch(`${baseUrl.replace(/\\/$/, '')}/api/v1/system/info`, {
      cache: 'no-store',
      signal: AbortSignal.timeout(2500),
      headers: { accept: 'application/json' },
    });
    if (!response.ok) return { connected: false, reason: 'unavailable' };

    const info = (await response.json()) as SystemInfo;
    if (!info.name || !info.api_version || !info.runtime) {
      return { connected: false, reason: 'unavailable' };
    }
    return { connected: true, info };
  } catch {
    return { connected: false, reason: 'unavailable' };
  }
}
