import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';
import { loadTenantResource, tenantHref, type SiteRecord } from '@/lib/network';

type SearchParams = Promise<{ tenant?: string }>;

export default async function NetworkSitesPage({ searchParams }: { searchParams: SearchParams }) {
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
    return (
      <main className="inventory-page">
        <header className="inventory-header">
          <div><span className="section-index">NETWORK / LOCATIONS</span><h1>SELECT TENANT</h1><p>Choose an organization scope before loading sites.</p></div>
          <Link className="inventory-back" href="/">COMMAND CENTER ↗</Link>
        </header>
        {tenants.length === 0 ? <p className="inventory-alert">No active tenant membership is available for this account.</p> : (
          <form className="tenant-picker" method="get">
            <label htmlFor="tenant">ACTIVE TENANT</label>
            <select id="tenant" name="tenant" required defaultValue="">
              <option value="" disabled>Select a tenant…</option>
              {tenants.map(item => <option value={item.tenant_id} key={item.tenant_id}>{item.organization_name} / {item.tenant_name} · {item.role}</option>)}
            </select>
            <button type="submit">OPEN SITE REGISTER ↗</button>
          </form>
        )}
      </main>
    );
  }

  const result = await loadTenantResource<SiteRecord[]>('/api/v1/network/sites', token, tenant.tenant_id);
  return (
    <main className="inventory-page">
      <header className="inventory-header">
        <div>
          <span className="section-index">NETWORK / LOCATIONS / SITES</span>
          <h1>SITE REGISTER</h1>
          <p>{tenant.organization_name} / {tenant.tenant_name}</p>
        </div>
        <div className="inventory-header-actions">
          <Link className="inventory-back" href={tenantHref('/network/pops', tenant.tenant_id)}>POP REGISTER ↗</Link>
          <Link className="inventory-back" href={tenantHref('/network/devices', tenant.tenant_id)}>DEVICES ↗</Link>
          <Link className="inventory-back" href="/">COMMAND CENTER ↗</Link>
        </div>
      </header>

      <section className="inventory-summary">
        <div><span>SITES IN TENANT</span><strong>{result.kind === 'ok' && Array.isArray(result.data) ? result.data.length : '—'}</strong></div>
        <div><span>ACTIVE TENANT ROLE</span><strong>{tenant.role.toUpperCase()}</strong></div>
        <div><span>DATA SOURCE</span><strong>LOCO API</strong></div>
      </section>

      {result.kind === 'unavailable' ? <p className="inventory-alert">Site inventory API is unavailable. No cached or sample sites are shown.</p>
        : result.kind === 'forbidden' ? <p className="inventory-alert">This account is not permitted to read this tenant's site inventory.</p>
        : result.kind !== 'ok' || !Array.isArray(result.data) ? <p className="inventory-alert">Site inventory could not be loaded.</p>
        : result.data.length === 0 ? (
          <section className="inventory-empty"><span className="section-index">NO LOCATIONS REGISTERED</span><h2>NO SITES FOUND.</h2><p>Create a site through the authorized network API. This screen is read-only and does not invent location records.</p></section>
        ) : (
          <div className="inventory-table-wrap">
            <table className="inventory-table">
              <thead><tr><th>SITE</th><th>SLUG</th><th>REGION UUID</th><th>TYPE</th><th>TIMEZONE</th><th>STATUS</th><th>DETAIL</th></tr></thead>
              <tbody>{result.data.map(site => (
                <tr key={site.id}>
                  <td><strong>{site.name}</strong><small>{site.id}</small></td>
                  <td><code>{site.slug}</code></td>
                  <td>{site.region_id ?? 'NOT ASSIGNED'}</td>
                  <td>{site.site_type.toUpperCase()}</td>
                  <td>{site.timezone}</td>
                  <td><span className="inventory-state">{site.status.toUpperCase()}</span></td>
                  <td><Link className="inventory-row-link" href={tenantHref(`/network/sites/${encodeURIComponent(site.id)}`, tenant.tenant_id)}>OPEN ↗</Link></td>
                </tr>
              ))}</tbody>
            </table>
            <div className="inventory-pagination"><span>{result.data.length} SITES · API LIMIT 500</span><span>READ-ONLY REGISTER</span></div>
          </div>
        )}
    </main>
  );
}
