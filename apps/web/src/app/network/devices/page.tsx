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

type PageState =
  | { kind: 'ready'; devices: Device[] }
  | { kind: 'unavailable' }
  | { kind: 'failed' };

function apiBaseUrl(): string | null {
  const value = process.env.API_INTERNAL_URL?.trim();
  if (!value) return null;
  return value.endsWith('/') ? value.slice(0, -1) : value;
}

async function loadDevices(token: string, tenantId: string): Promise<PageState> {
  const baseUrl = apiBaseUrl();
  if (!baseUrl) return { kind: 'unavailable' };

  try {
    const response = await fetch(`${baseUrl}/api/v1/network/devices`, {
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
    const devices = (await response.json()) as Device[];
    if (!Array.isArray(devices)) return { kind: 'failed' };
    return { kind: 'ready', devices };
  } catch {
    return { kind: 'unavailable' };
  }
}

export default async function NetworkDevicesPage({
  searchParams,
}: {
  searchParams: Promise<{ tenant?: string }>;
}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('eisp_session')?.value;
  if (!token) redirect('/login');

  const session = await getCurrentUser(token);
  if (!session.authenticated) {
    if (session.reason === 'unauthorized') redirect('/login');
    return <main className="inventory-page"><p className="inventory-alert">Authentication service is unavailable.</p></main>;
  }

  const { tenant: requestedTenant } = await searchParams;
  const tenants = session.current.tenants;
  const selectedTenant =
    tenants.find(tenant => tenant.tenant_id === requestedTenant) ??
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

  const state = await loadDevices(token, selectedTenant.tenant_id);

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
        <div><span>REGISTERED DEVICES</span><strong>{state.kind === 'ready' ? state.devices.length : '—'}</strong></div>
        <div><span>ACTIVE TENANT ROLE</span><strong>{selectedTenant.role.toUpperCase()}</strong></div>
        <div><span>DATA SOURCE</span><strong>LOCO API / LIVE QUERY</strong></div>
      </section>

      {state.kind === 'unavailable' ? (
        <p className="inventory-alert">The network inventory API is unavailable. No cached or sample devices are shown.</p>
      ) : state.kind === 'failed' ? (
        <p className="inventory-alert">The request was rejected or returned an invalid response. Confirm tenant membership and API access.</p>
      ) : state.devices.length === 0 ? (
        <section className="inventory-empty">
          <span className="section-index">NO ASSETS REGISTERED</span>
          <h2>THE INVENTORY IS EMPTY.</h2>
          <p>Devices will appear here after they are added through the authorized inventory API. Discovery and automatic polling are not enabled yet.</p>
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
              {state.devices.map(device => (
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
          <p className="inventory-footnote">Showing up to 500 devices. The current API uses a bounded list; cursor pagination is a separate delivery task.</p>
        </div>
      )}
    </main>
  );
}
