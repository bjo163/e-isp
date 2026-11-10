'use client';

import { FormEvent, useState } from 'react';
import { useRouter } from 'next/navigation';

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError('');

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });
      const result = (await response.json()) as { message?: string };
      if (!response.ok) {
        setError(result.message ?? 'Sign-in failed.');
        return;
      }
      router.replace('/');
      router.refresh();
    } catch {
      setError('Unable to reach the sign-in service.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="login-shell">
      <section className="login-panel">
        <a className="login-brand" href="/" aria-label="e-ISP home">e<span>·</span>ISP</a>
        <div className="eyebrow"><span className="eyebrow-line" /> SECURE OPERATOR ACCESS</div>
        <h1 className="login-title">ENTER THE<br /><em>COMMAND</em> FLOOR.</h1>
        <p className="login-description">Sign in with an administrator-provisioned account. Public self-registration is disabled.</p>
        <form className="login-form" onSubmit={submit}>
          <label htmlFor="email">EMAIL ADDRESS</label>
          <input id="email" type="email" autoComplete="username" required maxLength={254} value={email} onChange={event => setEmail(event.target.value)} placeholder="operator@isp.example" />
          <label htmlFor="password">PASSWORD</label>
          <input id="password" type="password" autoComplete="current-password" required minLength={12} maxLength={256} value={password} onChange={event => setPassword(event.target.value)} placeholder="12 characters minimum" />
          {error ? <p className="login-error" role="alert">{error}</p> : null}
          <button className="login-submit" type="submit" disabled={busy}>{busy ? 'AUTHENTICATING…' : 'SIGN IN'} <span>↗</span></button>
        </form>
        <div className="login-foot">ACCESS IS CONTROLLED BY YOUR ORGANIZATION.</div>
      </section>
      <div className="login-art" aria-hidden="true"><div className="login-orbit one" /><div className="login-orbit two" /><div className="login-core">e<small>ISP</small></div><span className="login-node a">IDENTITY</span><span className="login-node b">TENANT</span><span className="login-node c">OPERATIONS</span></div>
    </main>
  );
}
