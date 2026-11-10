import Link from 'next/link';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/system';
import { loadTenantResource, tenantHref, type PopRecord, type SiteRecord } from '@/lib/network';

export default async function PopDetailPage({
  params,
  searchParams,
}: {
  params: Promise<{ pop_id: string }>;
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

  const [{ pop_id: popId }, query] = await Promise.all([params, searchParams]);
  const tenants = session.current.tenants;
  const tenant = tenants.find(item => item.tenant_id === query.tenant) ?? (tenants.length === 1 ? tenants[0] : null);
  if (!tenant) {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / POPS</span><h1>SELECT TENANT</h1><p>Choose a tenant before opening a POP record.</p></div><Link className="inventory-back" href="/">COMMAND CENTER ↗</Link></header>{tenants.length === 0 ? <p className="inventory-alert">No active tenant membership is available.</p> : <form className="tenant-picker" method="get"><label htmlFor="tenant">ACTIVE TENANT</label><select id="tenant" name="tenant" required defaultValue=""><option value="" disabled>Select a tenant…</option>{tenants.map(item => <option key={item.tenant_id} value={item.tenant_id}>{item.organization_name} / {item.tenant_name}</option>)}</select><button type="submit">OPEN POP ↗</button></form>}</main>;
  }

  const popResult = await loadTenantResource<PopRecord>(`/api/v1/network/pops/${encodeURIComponent(popId)}`, token, tenant.tenant_id);
  if (popResult.kind === 'not-found') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / POP DETAIL</span><h1>POP NOT FOUND.</h1><p>The POP does not exist in the selected tenant scope.</p></div><Link className="inventory-back" href={tenantHref('/network/pops', tenant.tenant_id)}>BACK TO POPS ↗</Link></header></main>;
  }
  if (popResult.kind !== 'ok') {
    return <main className="inventory-page"><header className="inventory-header"><div><span className="section-index">NETWORK / POP DETAIL</span><h1>DETAIL UNAVAILABLE.</h1><p>POP metadata could not be loaded.</p></div><Link className="inventory-back" href={tenantHref('/network/pops', tenant.tenant_id)}>BACK TO POPS ↗</Link></header><p className="inventory-alert">{popResult.kind === 'forbidden' ? 'Access to this tenant was denied.' : 'The network API is unavailable.'}</p></main>;
  }

  const siteResult = await loadTenantResource<SiteRecord>(`/api/v1/network/sites/${encodeURIComponent(popResult.data.site_id)}`, token, tenant.tenant_id);
  return (
    <main className="inventory-page">
      <header className="inventory-header"><div><span className="section-index">NETWORK / LOCATIONS / POP DETAIL</span><h1>{popResult.data.name.toUpperCase()}</h1><p>{tenant.organization_name} / {tenant.tenant_name} · {popResult.data.slug}</p></div><div className="inventory-header-actions"><Link className="inventory-back" href={tenantHref('/network/pops', tenant.tenant_id)}>ALL POPS ↗</Link><Link className="inventory-back" href={tenantHref('/network/sites', tenant.tenant_id)}>ALL SITES ↗</Link></div></header>
      <section className="inventory-summary"><div><span>POP TYPE</span><strong>{popResult.data.pop_type.toUpperCase()}</strong></div><div><span>LIFECYCLE</span><strong>{popResult.data.status.toUpperCase()}</strong></div><div><span>PARENT SITE</span><strong>{siteResult.kind === 'ok' ? siteResult.data.name.toUpperCase() : 'UNAVAILABLE'}</strong></div></section>
      <section className="location-detail-grid">
        <article className="location-panel"><span className="section-index">POP IDENTITY</span><dl><div><dt>PUBLIC UUID</dt><dd>{popResult.data.id}</dd></div><div><dt>SLUG</dt><dd>{popResult.data.slug}</dd></div><div><dt>POP TYPE</dt><dd>{popResult.data.pop_type}</dd></div><div><dt>STATUS</dt><dd>{popResult.data.status}</dd></div></dl></article>
        <article className="location-panel"><span className="section-index">PARENT SITE</span>{siteResult.kind === 'ok' ? <><h2 className="location-parent-title">{siteResult.data.name}</h2><p className="location-muted">{siteResult.data.site_type.toUpperCase()} · {siteResult.data.timezone}</p><Link className="inventory-back" href={tenantHref(`/network/sites/${encodeURIComponent(siteResult.data.id)}`, tenant.tenant_id)}>OPEN SITE ↗</Link></> : <p className="location-muted">Parent site details are currently unavailable. The POP record itself was resolved within the selected tenant.</p>}</article>
      </section>
    </main>
  );
}
