export interface SystemInfo {
  name: string;
  version: string;
  api_version: string;
  status: string;
  runtime: string;
}

export interface CurrentUser {
  user: { id: string; email: string; name: string };
  tenants: Array<{
    tenant_id: string;
    tenant_name: string;
    tenant_slug: string;
    organization_id: string;
    organization_name: string;
    role: string;
  }>;
}

export type ApiConnection =
  | { connected: true; info: SystemInfo }
  | { connected: false; reason: 'unconfigured' | 'unavailable' };

export type SessionResult =
  | { authenticated: true; current: CurrentUser }
  | { authenticated: false; reason: 'unauthorized' | 'unavailable' };

function apiBaseUrl(): string | null {
  const configured = process.env.API_INTERNAL_URL?.trim();
  if (!configured) return null;
  return configured.endsWith('/') ? configured.slice(0, -1) : configured;
}

export async function getApiConnection(): Promise<ApiConnection> {
  const baseUrl = apiBaseUrl();
  if (!baseUrl) return { connected: false, reason: 'unconfigured' };

  try {
    const response = await fetch(`${baseUrl}/api/v1/system/info`, {
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

export async function getCurrentUser(token: string): Promise<SessionResult> {
  const baseUrl = apiBaseUrl();
  if (!baseUrl) return { authenticated: false, reason: 'unavailable' };

  try {
    const response = await fetch(`${baseUrl}/api/v1/auth/me`, {
      cache: 'no-store',
      signal: AbortSignal.timeout(3000),
      headers: { accept: 'application/json', authorization: `Bearer ${token}` },
    });
    if (response.status === 401 || response.status === 403) {
      return { authenticated: false, reason: 'unauthorized' };
    }
    if (!response.ok) return { authenticated: false, reason: 'unavailable' };

    const current = (await response.json()) as CurrentUser;
    if (!current.user?.id || !Array.isArray(current.tenants)) {
      return { authenticated: false, reason: 'unavailable' };
    }
    return { authenticated: true, current };
  } catch {
    return { authenticated: false, reason: 'unavailable' };
  }
}
