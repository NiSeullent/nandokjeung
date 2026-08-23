import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: '난독증 — 정신뼝! 난독화',
  description: 'Fortress 난독화 + 순환 해독 웹 UI',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="ko">
      <body>{children}</body>
    </html>
  );
}
