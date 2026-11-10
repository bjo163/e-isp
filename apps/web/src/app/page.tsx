import { getApiConnection } from '@/lib/system';

const modules = [
  { code: 'NET', title: 'Network inventory', detail: 'Devices, interfaces, sites and POPs', state: 'Foundation' },
  { code: 'OBS', title: 'Observability', detail: 'Telemetry, traffic and freshness', state: 'Planned' },
  { code: 'TOP', title: 'Topology intelligence', detail: 'Physical, logical and service dependency', state: 'Planned' },
  { code: 'ISP', title: 'Service assurance', detail: 'Subscribers, circuits and SLA', state: 'Planned' },
  { code: 'FTTH', title: 'Fiber operations', detail: 'OLT, PON, ONT and GIS', state: 'Planned' },
  { code: 'AUTO', title: 'Controlled automation', detail: 'Change, approval and verification', state: 'Planned' },
];

export default async function Home() {
  const api = await getApiConnection();

  return (
    <main className="shell">
      <aside className="rail" aria-label="Primary navigation">
        <div className="brand-mark">e<span>·</span></div>
        <div className="rail-separator" />
        <a className="rail-link rail-link-active" href="#overview" aria-label="Command center">⌂</a>
        <a className="rail-link" href="#modules" aria-label="Modules">▦</a>
        <a className="rail-link" href="https://github.com/bjo163/e-isp/issues/110" aria-label="Execution backlog">☷</a>
        <div className="rail-spacer" />
        <a className="rail-link" href="https://github.com/bjo163/e-isp" aria-label="GitHub repository">↗</a>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div className="crumb"><span>OPERATIONS</span><b>/</b><strong>COMMAND CENTER</strong></div>
          <div className="topbar-right">
            <span className="environment">SELF-HOSTED</span>
            <span className="operator"><span className="operator-dot" /> OPERATOR CONSOLE</span>
          </div>
        </header>

        <section className="hero" id="overview">
          <div className="hero-copy">
            <div className="eyebrow"><span className="eyebrow-line" /> NETWORK OPERATIONS CENTER <span className="eyebrow-version">FOUNDATION 0.1</span></div>
            <h1>SEE THE<br /><em>WHOLE</em> NETWORK.</h1>
            <p className="hero-subtitle">Know the network. Protect the service. Automate the operation.</p>
            <div className="hero-note">ONE OPERATIONAL PICTURE — FROM CORE TO CUSTOMER EDGE.</div>
          </div>
          <div className="hero-art" aria-hidden="true">
            <div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="orbit orbit-three" />
            <div className="core-node"><span>e</span><small>ISP</small></div>
            <span className="node node-a">CORE</span><span className="node node-b">POP-01</span><span className="node node-c">EDGE</span>
            <span className="art-cross cross-a">+</span><span className="art-cross cross-b">+</span>
          </div>
        </section>

        <section className="status-strip" aria-label="Platform status">
          <div className="status-cell">
            <span className="status-label">CONTROL PLANE</span>
            <strong className={api.connected ? 'state-ok' : 'state-warn'}><i /> {api.connected ? 'API CONNECTED' : 'API NOT CONNECTED'}</strong>
          </div>
          <div className="status-cell">
            <span className="status-label">API VERSION</span>
            <strong>{api.connected ? api.info.api_version.toUpperCase() : 'WAITING'}</strong>
          </div>
          <div className="status-cell">
            <span className="status-label">COLLECTOR TELEMETRY</span>
            <strong className="state-muted"><i /> NOT CONFIGURED</strong>
          </div>
          <div className="status-cell status-cell-last">
            <span className="status-label">OPERATIONAL DATA</span>
            <strong className="state-muted">NO LIVE DATA YET</strong>
          </div>
        </section>

        <section className="section-heading" id="modules">
          <div><span className="section-index">01 / PLATFORM MAP</span><h2>Operational domains</h2></div>
          <span className="section-caption">MODULES ACTIVATE AS THEIR DELIVERY GATES PASS</span>
        </section>

        <section className="module-grid" aria-label="Platform domains">
          {modules.map((module, index) => (
            <article className="module-card" key={module.code}>
              <div className="module-card-top"><span className="module-code">{module.code}</span><span className="module-number">{String(index + 1).padStart(2, '0')}</span></div>
              <h3>{module.title}</h3>
              <p>{module.detail}</p>
              <div className="module-card-bottom"><span className={module.state === 'Foundation' ? 'module-state state-active' : 'module-state'}>{module.state}</span><span className="module-arrow">↗</span></div>
            </article>
          ))}
        </section>

        <section className="bottom-line">
          <div><span className="section-index">02 / DELIVERY TRACKER</span><h2>Build in evidence.<br /><em>Operate with confidence.</em></h2></div>
          <div className="delivery-copy"><p>The platform is being delivered as small, verifiable engineering units. No sample values are presented as production telemetry. Every operational capability must pass its issue acceptance criteria before it is marked active.</p><a href="https://github.com/bjo163/e-isp/issues/110">OPEN MASTER EXECUTION BOARD <span>↗</span></a></div>
        </section>

        <footer className="footer"><span>e-ISP / NETWORK OPERATIONS PLATFORM</span><span>NETWORK-FIRST · SERVICE-AWARE · AUTOMATION-DRIVEN</span></footer>
      </div>
    </main>
  );
}
