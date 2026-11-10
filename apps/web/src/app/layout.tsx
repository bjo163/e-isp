import type { Metadata, Viewport } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'e-ISP — Network Operations Center',
  description: 'Network operations, service assurance, and automation platform for ISP and telecom operators.',
};

export const viewport: Viewport = {
  themeColor: '#0b0d10',
  colorScheme: 'dark light',
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
