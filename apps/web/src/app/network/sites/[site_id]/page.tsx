import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';
import { loadTenantResource, tenantHref, type SiteRecord, type PopRecord } from '@/lib/network';

export default async function SiteDetailPage({
  params,
  searchParams,
}: {
  params: Promise<{ site_id: string }>;
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

  const [{ site_id: siteId }, query] = await Promise.all([params, searchParams]);
  const tenants = session.current.tenants;
  const tenant = tenants.find(item => item.tenant_id === query.tenant) ?? (tenants.length === 1 ? tenants[0] : null);
  if (!tenant) {
    return (
      <main className="inventory-page">
        <header className="inventory-header"><div><span className="section-index">NETWORK / SITES</span><h1>SELECT TENANT</h1><p>Choose a tenant before opening a site record.</p></div><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></header>
        {tenants.length === 0 ? <p className="inventory-alert">No active tenant membership is available.</p> : (
          <form className="tenant-picker" method="get"><label htmlFor="tenant">ACTIVE TENANT</label><select id="tenant" name="tenant" required defaultValue=""><option value="" disabled>Select a tenant…</option>{tenants.map(item => <option key={item.tenant_id} value={item.tenant_id}>{item.organization_name} / {item.tenant_name}</option>)}</select><button type="submit">OPEN SITE ↗</button></form>
        )}
      </main>
    );
  }

  const siteResult = await loadTenantResource<SiteRecord>(`/api/v1/network/sites/${encodeURIComponent(siteId)}`, token, tenant.tenant_id);
  if (siteResult.kind === 'not-found') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / SITE DETAIL</span><h1>SITE NOT FOUND.</h1><p>The site does not exist in the selected tenant scope.</p></div><Link className="inventory-back" href={tenantHref('/network/sites', tenant.tenant_id)}>BACK TO SITES ↗</Link></header></main>;
  }
  if (siteResult.kind !== 'ok') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / SITE DETAIL</span><h1>DETAIL UNAVAILABLE.</h1><p>Site metadata could not be loaded.</p></div><Link className="inventory-back" href={tenantHref('/network/sites', tenant.tenant_id)}>BACK TO SITES ↗</Link></header><p className="inventory-alert">{siteResult.kind === 'forbidden' ? 'Access to this tenant was denied.' : 'The network API is unavailable or returned an invalid response.'}</p></main>;
  }

  const popResult = await loadTenantResource<PopRecord[]>('/api/v1/network/pops', token, tenant.tenant_id);
  const childPops = popResult.kind === 'ok' && Array.isArray(popResult.data) ? popResult.data.filter(pop => pop.site_id === siteResult.data.id) : null;
  const site = siteResult.data;

  return (
    <main className="inventory-page">
      <header className="inventory-header">
        <div><span className="section-index">NETWORK / LOCATIONS / SITE DETAIL</span><h1>{site.name.toUpperCase()}</h1><p>{tenant.organization_name} / {tenant.tenant_name} · {site.slug}</p></div>
        <div className="inventory-header-actions"><Link className="inventory-back" href={tenantHref('/network/sites', tenant.tenant_id)}>ALL SITES ↗</Link><Link className="inventory-back" href={tenantHref('/network/pops', tenant.tenant_id)}>ALL POPS ↗</Link></div>
      </header>
      <section className="inventory-summary">
        <div><span>SITE TYPE</span><strong>{site.site_type.toUpperCase()}</strong></div>
        <div><span>TIMEZONE</span><strong>{site.timezone}</strong></div>
        <div><span>LIFECYCLE</span><strong>{site.status.toUpperCase()}</strong></div>
      </section>
      <section className="location-detail-grid">
        <article className="location-panel"><span className="section-index">SITE IDENTITY</span><dl>
          <div><dt>PUBLIC UUID</dt><dd>{site.id}</dd></div>
          <div><dt>SLUG</dt><dd>{site.slug}</dd></div>
          <div><dt>REGION</dt><dd>{site.region_id ?? 'NOT ASSIGNED'}</dd></div>
          <div><dt>TENANT</dt><dd>{tenant.tenant_name}</dd></div>
        </dl></article>
        <article className="location-panel"><span className="section-index">CHILD POPS</span>
          {childPops === null ? <p className="location-muted">POP list is temporarily unavailable.</p> : childPops.length === 0 ? <p className="location-muted">No POP is currently registered under this site.</p> : (
            <ul className="location-child-list">{childPops.map(pop => <li key={pop.id}><div><strong>{pop.name}</strong><small>{pop.pop_type.toUpperCase()} · {pop.slug}</small></div><Link href={tenantHref(`/network/pops/${encodeURIComponent(pop.id)}`, tenant.tenant_id)}>OPEN ↗</Link></li>)}</ul>
          )}
        </article>
      </section>
    </main>
  );
}
