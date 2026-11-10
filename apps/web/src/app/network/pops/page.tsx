import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';
import { loadTenantResource, tenantHref, type PopRecord, type SiteRecord } from '@/lib/network';

export default async function NetworkPopsPage({ searchParams }: { searchParams: Promise<{ tenant?: string }> }) {
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
  const tenant = tenants.find(item => item.tenant_id === params.tenant) ?? (tenants.length === 1 ? tenants[0] : null);
  if (!tenant) {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / LOCATIONS</span><h1>SELECT TENANT</h1><p>Choose an organization scope before loading POPs.</p></div><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></header>{tenants.length === 0 ? <p className="inventory-alert">No active tenant membership is available.</p> : <form className="tenant-picker" method="get"><label htmlFor="tenant">ACTIVE TENANT</label><select id="tenant" name="tenant" required defaultValue=""><option value="" disabled>Select a tenant…</option>{tenants.map(item => <option key={item.tenant_id} value={item.tenant_id}>{item.organization_name} / {item.tenant_name} · {item.role}</option>)}</select><button type="submit">OPEN POP REGISTER ↗</button></form>}</main>;
  }

  const [popResult, siteResult] = await Promise.all([
    loadTenantResource<PopRecord[]>('/api/v1/network/pops', token, tenant.tenant_id),
    loadTenantResource<SiteRecord[]>('/api/v1/network/sites', token, tenant.tenant_id),
  ]);
  const sites = siteResult.kind === 'ok' && Array.isArray(siteResult.data) ? new Map(siteResult.data.map(site => [site.id, site.name])) : null;

  return (
    <main className="inventory-page">
      <header className="inventory-header"><div><span className="section-index">NETWORK / LOCATIONS / POPS</span><h1>POP REGISTER</h1><p>{tenant.organization_name} / {tenant.tenant_name}</p></div><div className="inventory-header-actions"><Link className="inventory-back" href={tenantHref('/network/sites', tenant.tenant_id)}>SITE REGISTER ↗</Link><Link className="inventory-back" href={tenantHref('/network/devices', tenant.tenant_id)}>DEVICES ↗</Link><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></div></header>
      <section className="inventory-summary"><div><span>POPS IN TENANT</span><strong>{popResult.kind === 'ok' && Array.isArray(popResult.data) ? popResult.data.length : '—'}</strong></div><div><span>ACTIVE TENANT ROLE</span><strong>{tenant.role.toUpperCase()}</strong></div><div><span>DATA SOURCE</span><strong>LOCO API</strong></div></section>
      {popResult.kind === 'unavailable' || popResult.kind === 'not-found' ? <p className="inventory-alert">POP inventory API is unavailable. No sample records are displayed.</p>
        : popResult.kind === 'forbidden' ? <p className="inventory-alert">Access to this tenant's POP inventory was denied.</p>
        : popResult.kind !== 'ok' || !Array.isArray(popResult.data) ? <p className="inventory-alert">POP inventory could not be loaded.</p>
        : popResult.data.length === 0 ? <section className="inventory-empty"><span className="section-index">NO ACCESS LOCATIONS</span><h2>NO POPS FOUND.</h2><p>Register POP metadata through the authorized API. No network telemetry is implied by inventory existence.</p></section>
        : <div className="inventory-table-wrap"><table className="inventory-table"><thead><tr><th>POP</th><th>SLUG</th><th>SITE</th><th>TYPE</th><th>STATUS</th><th>DETAIL</th></tr></thead><tbody>{popResult.data.map(pop => <tr key={pop.id}><td><strong>{pop.name}</strong><small>{pop.id}</small></td><td><code>{pop.slug}</code></td><td>{sites?.get(pop.site_id) ?? 'SITE UNAVAILABLE'}</td><td>{pop.pop_type.toUpperCase()}</td><td><span className="inventory-state">{pop.status.toUpperCase()}</span></td><td><Link className="inventory-row-link" href={tenantHref(`/network/pops/${encodeURIComponent(pop.id)}`, tenant.tenant_id)}>OPEN ↗</Link></td></tr>)}</tbody></table><div className="inventory-pagination"><span>{popResult.data.length} POPS · API LIMIT 500</span><span>READ-ONLY REGISTER</span></div></div>}
    </main>
  );
}
