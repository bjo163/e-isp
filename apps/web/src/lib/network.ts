export interface RegionRecord {
  id: string;
  name: string;
  slug: string;
  timezone: string;
  status: string;
}

export interface SiteRecord {
  id: string;
  region_id: string | null;
  name: string;
  slug: string;
  site_type: string;
  timezone: string;
  status: string;
}

export interface PopRecord {
  id: string;
  site_id: string;
  name: string;
  slug: string;
  pop_type: string;
  status: string;
}

export interface DeviceRecord {
  id: string;
  inventory_key: string;
  name: string;
  hostname: string | null;
  management_address: string | null;
  serial_number: string | null;
  device_role: string;
  lifecycle_status: string;
  site_id: string | null;
  pop_id: string | null;
  vendor_id: string | null;
  device_model_id: string | null;
}

export interface DeviceInterfaceRecord {
  id: string;
  device_id: string;
  stable_key: string;
  if_index: number | null;
  name: string;
  description: string | null;
  interface_type: string;
  admin_state: string;
  operational_state: string;
}

export type NetworkApiResult<T> =
  | { kind: 'ok'; data: T }
  | { kind: 'not-found' }
  | { kind: 'forbidden' }
  | { kind: 'unavailable' }
  | { kind: 'invalid' };

function apiBaseUrl(): string | null {
  const value = process.env.API_INTERNAL_URL?.trim();
  if (!value) return null;
  return value.endsWith('/') ? value.slice(0, -1) : value;
}

export async function loadTenantResource<T>(
  path: string,
  token: string,
  tenantId: string,
): Promise<NetworkApiResult<T>> {
  const baseUrl = apiBaseUrl();
  if (!baseUrl) return { kind: 'unavailable' };

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      cache: 'no-store',
      signal: AbortSignal.timeout(5000),
      headers: {
        accept: 'application/json',
        authorization: `Bearer ${token}`,
        'x-tenant-id': tenantId,
      },
    });
    if (response.status === 401 || response.status === 403) return { kind: 'forbidden' };
    if (response.status === 404) return { kind: 'not-found' };
    if (!response.ok) return { kind: 'unavailable' };
    return { kind: 'ok', data: (await response.json()) as T };
  } catch {
    return { kind: 'unavailable' };
  }
}

export function tenantHref(path: string, tenantId: string): string {
  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}tenant=${encodeURIComponent(tenantId)}`;
}
