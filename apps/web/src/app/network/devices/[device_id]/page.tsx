import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';
import {
  loadTenantResource,
  tenantHref,
  type DeviceInterfaceRecord,
  type DeviceRecord,
} from '@/lib/network';

export default async function DeviceDetailPage({
  params,
  searchParams,
}: {
  params: Promise<{ device_id: string }>;
  searchParams: Promise<{ tenant?: string; tab?: string }>;
}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('eisp_session')?.value;
  if (!token) redirect('/login');

  const session = await getCurrentUser(token);
  if (!session.authenticated) {
    if (session.reason === 'unauthorized') redirect('/login');
    return <main className="inventory-page"><p className="inventory-alert">Authentication service is unavailable.</p></main>;
  }

  const [{ device_id: deviceId }, query] = await Promise.all([params, searchParams]);
  const tenants = session.current.tenants;
  const tenant = tenants.find(item => item.tenant_id === query.tenant) ?? (tenants.length === 1 ? tenants[0] : null);
  if (!tenant) {
    return (
      <main className="inventory-page">
        <header className="inventory-header"><div><span className="section-index">NETWORK / DEVICES</span><h1>SELECT TENANT</h1><p>Choose a tenant before opening device details.</p></div><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></header>
        {tenants.length === 0 ? <p className="inventory-alert">No active tenant membership is available.</p> : (
          <form className="tenant-picker" method="get"><label htmlFor="tenant">ACTIVE TENANT</label><select id="tenant" name="tenant" required defaultValue=""><option value="" disabled>Select a tenant…</option>{tenants.map(item => <option key={item.tenant_id} value={item.tenant_id}>{item.organization_name} / {item.tenant_name}</option>)}</select><button type="submit">OPEN DEVICE ↗</button></form>
        )}
      </main>
    );
  }

  const [deviceResult, interfaceResult] = await Promise.all([
    loadTenantResource<DeviceRecord>(`/api/v1/network/devices/${encodeURIComponent(deviceId)}`, token, tenant.tenant_id),
    loadTenantResource<DeviceInterfaceRecord[]>(`/api/v1/network/devices/${encodeURIComponent(deviceId)}/interfaces`, token, tenant.tenant_id),
  ]);

  if (deviceResult.kind === 'not-found') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / INVENTORY / DEVICE DETAIL</span><h1>DEVICE NOT FOUND.</h1><p>The device does not exist in the selected tenant scope.</p></div><Link className="inventory-back" href={tenantHref('/network/devices', tenant.tenant_id)}>BACK TO DEVICES ↗</Link></header></main>;
  }
  if (deviceResult.kind !== 'ok') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / INVENTORY / DEVICE DETAIL</span><h1>DETAIL UNAVAILABLE.</h1><p>Device metadata could not be loaded.</p></div><Link className="inventory-back" href={tenantHref('/network/devices', tenant.tenant_id)}>BACK TO DEVICES ↗</Link></header><p className="inventory-alert">{deviceResult.kind === 'forbidden' ? 'Access to this tenant was denied.' : 'The network API is unavailable.'}</p></main>;
  }

  const device = deviceResult.data;
  const interfaces = interfaceResult.kind === 'ok' && Array.isArray(interfaceResult.data) ? interfaceResult.data : null;
  const activeTab = query.tab === 'interfaces' ? 'interfaces' : 'summary';
  const summaryHref = tenantHref(`/network/devices/${encodeURIComponent(device.id)}`, tenant.tenant_id);
  const interfaceHref = tenantHref(`/network/devices/${encodeURIComponent(device.id)}?tab=interfaces`, tenant.tenant_id);

  return (
    <main className="inventory-page">
      <header className="inventory-header device-detail-heading">
        <div><span className="section-index">NETWORK / INVENTORY / DEVICE DETAIL</span><h1>{device.name.toUpperCase()}</h1><p>{tenant.organization_name} / {tenant.tenant_name} · {device.inventory_key}</p></div>
        <div className="inventory-header-actions"><Link className="inventory-back" href={tenantHref('/network/devices', tenant.tenant_id)}>DEVICE REGISTER ↗</Link><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></div>
      </header>

      <section className="inventory-summary">
        <div><span>DEVICE ROLE</span><strong>{device.device_role.toUpperCase()}</strong></div>
        <div><span>LIFECYCLE STATE</span><strong>{device.lifecycle_status.toUpperCase()}</strong></div>
        <div><span>INTERFACE RECORDS</span><strong>{interfaces === null ? '—' : interfaces.length}</strong></div>
      </section>

      <nav className="device-detail-tabs" aria-label="Device details">
        <Link className={activeTab === 'summary' ? 'device-detail-tab device-detail-tab-active' : 'device-detail-tab'} href={summaryHref} aria-current={activeTab === 'summary' ? 'page' : undefined}>SUMMARY</Link>
        <Link className={activeTab === 'interfaces' ? 'device-detail-tab device-detail-tab-active' : 'device-detail-tab'} href={interfaceHref} aria-current={activeTab === 'interfaces' ? 'page' : undefined}>INTERFACES</Link>
      </nav>

      {activeTab === 'summary' ? (
        <section className="device-detail-grid">
          <article className="location-panel"><span className="section-index">INVENTORY KEY</span><p className="device-detail-value">{device.inventory_key}</p></article>
          <article className="location-panel"><span className="section-index">HOSTNAME</span><p className="device-detail-value">{device.hostname ?? 'NOT SET'}</p></article>
          <article className="location-panel"><span className="section-index">MANAGEMENT ADDRESS</span><p className="device-detail-value">{device.management_address ?? 'NOT SET'}</p></article>
          <article className="location-panel"><span className="section-index">SERIAL NUMBER</span><p className="device-detail-value">{device.serial_number ?? 'NOT RECORDED'}</p></article>
          <article className="location-panel"><span className="section-index">SITE PUBLIC UUID</span><p className="device-detail-value">{device.site_id ?? 'NOT ASSIGNED'}</p></article>
          <article className="location-panel"><span className="section-index">POP PUBLIC UUID</span><p className="device-detail-value">{device.pop_id ?? 'NOT ASSIGNED'}</p></article>
        </section>
      ) : (
        interfaceResult.kind === 'unavailable' || interfaceResult.kind === 'forbidden' || interfaceResult.kind === 'not-found' || interfaces === null ? (
          <p className="inventory-alert">Interface inventory is unavailable. Device identity remains available; no interface state is inferred.</p>
        ) : interfaces.length === 0 ? (
          <section className="inventory-empty"><span className="section-index">NO INTERFACES REGISTERED</span><h2>NO INTERFACES FOUND.</h2><p>Interface inventory is empty for this device. Collector telemetry is not implied.</p></section>
        ) : (
          <div className="inventory-table-wrap">
            <table className="inventory-table">
              <thead><tr><th>STABLE KEY</th><th>INTERFACE</th><th>TYPE</th><th>IF INDEX</th><th>ADMIN STATE</th><th>OPERATIONAL STATE</th></tr></thead>
              <tbody>{interfaces.map(item => (
                <tr key={item.id}>
                  <td><code>{item.stable_key}</code></td>
                  <td><strong>{item.name}</strong><small>{item.description ?? 'NO DESCRIPTION'}</small></td>
                  <td>{item.interface_type.toUpperCase()}</td>
                  <td>{item.if_index ?? 'NOT OBSERVED'}</td>
                  <td><span className="inventory-state">{item.admin_state.toUpperCase()}</span></td>
                  <td><span className="inventory-state">{item.operational_state.toUpperCase()}</span></td>
                </tr>
              ))}</tbody>
            </table>
            <div className="inventory-pagination"><span>{interfaces.length} INTERFACES</span><span>UNKNOWN MEANS NO OBSERVATION, NOT HEALTHY</span></div>
          </div>
        )
      )}
    </main>
  );
}
