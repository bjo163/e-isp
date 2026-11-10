import { NextResponse } from 'next/server';

export const dynamic = 'force-dynamic';

interface LoginResponse {
  token: string;
  expires_in: number;
  user: { id: string; email: string; name: string };
}

export async function POST(request: Request) {
  const baseUrl = process.env.API_INTERNAL_URL?.trim();
  if (!baseUrl) {
    return NextResponse.json({ message: 'Authentication service is not configured.' }, { status: 503 });
  }

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ message: 'Invalid request body.' }, { status: 400 });
  }

  try {
    const upstream = await fetch(`${baseUrl.replace(/\/$/, '')}/api/v1/auth/login`, {
      method: 'POST',
      cache: 'no-store',
      signal: AbortSignal.timeout(5000),
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify(body),
    });

    if (!upstream.ok) {
      return NextResponse.json(
        { message: upstream.status === 401 ? 'Email or password is incorrect.' : 'Sign-in could not be completed.' },
        { status: upstream.status === 401 ? 401 : upstream.status >= 500 ? 502 : upstream.status },
      );
    }

    const result = (await upstream.json()) as Partial<LoginResponse>;
    if (!result.token || !result.user?.id || !result.expires_in) {
      return NextResponse.json({ message: 'Authentication service returned an invalid response.' }, { status: 502 });
    }

    const response = NextResponse.json({ user: result.user });
    response.cookies.set('eisp_session', result.token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: Math.min(Math.max(result.expires_in, 60), 86_400),
    });
    return response;
  } catch {
    return NextResponse.json({ message: 'Authentication service is unavailable.' }, { status: 502 });
  }
}
