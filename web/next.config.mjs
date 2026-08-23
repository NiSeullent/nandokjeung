/** @type {import('next').NextConfig} */
const isStaticExport = process.env.NEXT_OUTPUT === 'export';

const nextConfig = {
  ...(isStaticExport ? { output: 'export' } : {}),
  async rewrites() {
    if (isStaticExport) return [];
    const api = process.env.NANDOKJEUNG_API_URL || 'http://127.0.0.1:8787';
    return [{ source: '/backend/:path*', destination: `${api}/:path*` }];
  },
};

export default nextConfig;
