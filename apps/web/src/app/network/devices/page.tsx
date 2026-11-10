import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';

interface Device {
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

interface DevicePageInfo {
  limit: number;
  next_cursor: string | null;
  has_more: boolean;
}

interface DeviceListResponse {
  items: Device[];
  page: DevicePageInfo;
}

interface DeviceFilters {
  tenant?: string;
  q?: string;
  role?: string;
  lifecycle_status?: string;
  site_id?: string;
  pop_id?: string;
  after?: string;
  limit?: string;
}

type PageState =
  | { kind: 'ready'; result: DeviceListResponse }
  | { kind: 'unavailable' }
  | { kind: 'failed' };

function apiBaseUrl(): string | null {
  const value = process.env.API_INTERNAL_URL?.trim();
  if (!value) return null;
  return value.endsWith('/') ? value.slice(0, -1) : value;
}

function queryString(filters: DeviceFilters): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(filters)) {
    if (value?.trim()) params.set(key, value.trim());
  }
  return params.toString();
}

async function loadDevices(
  token: string,
  tenantId: string,
  filters: DeviceFilters,
): Promise<PageState> {
  const baseUrl = apiBaseUrl();
  if (!baseUrl) return { kind: 'unavailable' };

  const query = queryString(filters);
  const url = `${baseUrl}/api/v1/network/devices${query ? `?${query}` : ''}`;

  try {
    const response = await fetch(url, {
      cache: 'no-store',
      signal: AbortSignal.timeout(5000),
      headers: {
        accept: 'application/json',
        authorization: `Bearer ${token}`,
        'x-tenant-id': tenantId,
      },
    });
    if (response.status === 401 || response.status === 403) {
      return { kind: 'failed' };
    }
    if (!response.ok) return { kind: 'unavailable' };
    const result = (await response.json()) as DeviceListResponse;
    if (
      !result ||
      !Array.isArray(result.items) ||
      !result.page ||
      typeof result.page.has_more !== 'boolean' ||
      typeof result.page.limit !== 'number'
    ) {
      return { kind: 'failed' };
    }
    return { kind: 'ready', result };
  } catch {
    return { kind: 'unavailable' };
  }
}

export default async function NetworkDevicesPage({
  searchParams,
}: {
  searchParams: Promise<{
    tenant?: string;
    q?: string;
    role?: string;
    lifecycle_status?: string;
    site_id?: string;
    pop_id?: string;
    after?: string;
    limit?: string;
  }>;
}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('eisp_session')?.value;
  if (!token) redirect('/login');

  const session = await getCurrentUser(token);
  if (!session.authenticated) {
    if (session.reason === 'unauthorized') redirect('/login');
    return <main className="inventory-page"><p className="inventory-alert">Authentication service is unavailable.</p></main>;
  }

  const params = await searchParams;
  const tenants = session.current.tenants;
  const selectedTenant =
    tenants.find(tenant => tenant.tenant_id === params.tenant) ??
    (tenants.length === 1 ? tenants[0] : null);

  if (!selectedTenant) {
    return (
      <main className="inventory-page">
        <header className="inventory-header">
          <div>
            <span className="section-index">NETWORK / INVENTORY</span>
            <h1>SELECT TENANT</h1>
            <p>Choose an organization scope before loading network assets.</p>
          </div>
          <Link className="inventory-back" href="/">COMMAND CENTER ↗</Link>
        </header>
        {tenants.length === 0 ? (
          <p className="inventory-alert">This account has no active tenant membership. Ask an organization owner to grant access.</p>
        ) : (
          <form className="tenant-picker" method="get">
            <label htmlFor="tenant">ACTIVE TENANT</label>
            <select id="tenant" name="tenant" required defaultValue="">
              <option value="" disabled>Select a tenant…</option>
              {tenants.map(tenant => (
                <option value={tenant.tenant_id} key={tenant.tenant_id}>
                  {tenant.organization_name} / {tenant.tenant_name} · {tenant.role}
                </option>
              ))}
            </select>
            <button type="submit">OPEN NETWORK INVENTORY ↗</button>
          </form>
        )}
      </main>
    );
  }

  const filters: DeviceFilters = {
    q: params.q,
    role: params.role,
    lifecycle_status: params.lifecycle_status,
    site_id: params.site_id,
    pop_id: params.pop_id,
    after: params.after,
    limit: params.limit ?? '50',
  };
  const state = await loadDevices(token, selectedTenant.tenant_id, filters);
  const nextHref = state.kind === 'ready' && state.result.page.next_cursor
    ? `/network/devices?${queryString({
        ...filters,
        tenant: selectedTenant.tenant_id,
        after: state.result.page.next_cursor,
      })}`
    : null;

  return (
    <main className="inventory-page">
      <header className="inventory-header">
        <div>
          <span className="section-index">NETWORK / INVENTORY / DEVICES</span>
          <h1>DEVICE REGISTER</h1>
          <p>{selectedTenant.organization_name} / {selectedTenant.tenant_name}</p>
        </div>
        <div className="inventory-header-actions">
          {tenants.length > 1 ? <Link className="inventory-back" href="/network/devices">CHANGE TENANT</Link> : null}
          <Link className="inventory-back" href="/">COMMAND CENTER ↗</Link>
        </div>
      </header>

      <section className="inventory-summary">
        <div><span>DEVICES IN THIS PAGE</span><strong>{state.kind === 'ready' ? state.result.items.length : '—'}</strong></div>
        <div><span>ACTIVE TENANT ROLE</span><strong>{selectedTenant.role.toUpperCase()}</strong></div>
        <div><span>PAGE SIZE</span><strong>{state.kind === 'ready' ? state.result.page.limit : '—'}</strong></div>
      </section>

      <form className="inventory-filters" method="get">
        <input type="hidden" name="tenant" value={selectedTenant.tenant_id} />
        <input type="hidden" name="limit" value={filters.limit} />
        <label className="inventory-filter-search">
          <span>SEARCH DEVICES</span>
          <input name="q" type="search" maxLength={120} placeholder="Name, inventory key, hostname, IP…" defaultValue={params.q ?? ''} />
        </label>
        <label>
          <span>ROLE</span>
          <select name="role" defaultValue={params.role ?? ''}>
            <option value="">ALL ROLES</option>
            <option value="router">ROUTER</option>
            <option value="switch">SWITCH</option>
            <option value="firewall">FIREWALL</option>
            <option value="olt">OLT</option>
            <option value="server">SERVER</option>
            <option value="wireless">WIRELESS</option>
            <option value="other">OTHER</option>
          </select>
        </label>
        <label>
          <span>LIFECYCLE</span>
          <select name="lifecycle_status" defaultValue={params.lifecycle_status ?? ''}>
            <option value="">ALL STATES</option>
            <option value="planned">PLANNED</option>
            <option value="active">ACTIVE</option>
            <option value="maintenance">MAINTENANCE</option>
            <option value="retired">RETIRED</option>
          </select>
        </label>
        <label>
          <span>SITE UUID</span>
          <input name="site_id" placeholder="Optional site UUID" defaultValue={params.site_id ?? ''} />
        </label>
        <label>
          <span>POP UUID</span>
          <input name="pop_id" placeholder="Optional POP UUID" defaultValue={params.pop_id ?? ''} />
        </label>
        <button type="submit">APPLY FILTERS ↗</button>
        <Link className="inventory-filter-reset" href={`/network/devices?tenant=${encodeURIComponent(selectedTenant.tenant_id)}`}>RESET</Link>
      </form>

      {state.kind === 'unavailable' ? (
        <p className="inventory-alert">The network inventory API is unavailable. No cached or sample devices are shown.</p>
      ) : state.kind === 'failed' ? (
        <p className="inventory-alert">The request was rejected or returned an invalid response. Confirm tenant membership and filter values.</p>
      ) : state.result.items.length === 0 ? (
        <section className="inventory-empty">
          <span className="section-index">NO MATCHING ASSETS</span>
          <h2>NO DEVICES FOUND.</h2>
          <p>No device matches the current tenant and filters. Clear filters or add devices through the authorized inventory API.</p>
        </section>
      ) : (
        <div className="inventory-table-wrap">
          <table className="inventory-table">
            <thead>
              <tr>
                <th>INVENTORY KEY</th>
                <th>DEVICE</th>
                <th>ROLE</th>
                <th>MANAGEMENT ADDRESS</th>
                <th>STATUS</th>
                <th>SERIAL</th>
              </tr>
            </thead>
            <tbody>
              {state.result.items.map(device => (
                <tr key={device.id}>
                  <td><code>{device.inventory_key}</code></td>
                  <td><strong>{device.name}</strong><small>{device.hostname ?? 'HOSTNAME NOT SET'}</small></td>
                  <td>{device.device_role.toUpperCase()}</td>
                  <td>{device.management_address ?? 'NOT SET'}</td>
                  <td><span className="inventory-state">{device.lifecycle_status.toUpperCase()}</span></td>
                  <td>{device.serial_number ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="inventory-pagination">
            <span>{state.result.items.length} DEVICES · SORTED BY INVENTORY KEY</span>
            {state.result.page.has_more && nextHref ? <Link className="inventory-back" href={nextHref}>NEXT PAGE ↗</Link> : <span>END OF RESULTS</span>}
          </div>
        </div>
      )}
    </main>
  );
}
