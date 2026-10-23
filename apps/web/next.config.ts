import path from 'node:path';
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'export',
  // 家目录里可能躺着一个无关的 package-lock.json，Next 据此会把用户主目录
  // （C:\Users\<user>）推断成 workspace root，导致 dev 时监视整个用户目录。
  // 显式钉回仓库根，避免这个副作用。
  outputFileTracingRoot: path.resolve(process.cwd(), '../..'),
  experimental: {
    // 静态导出时 Next 会把 /404 和 /404.html 当成两条导出路径，二者并发写同一个
    // out/404.html。在 Windows 上同时新建同一文件会稳定报
    // EPERM: operation not permitted, open '...out\404.html'，导致 next build 失败。
    // 串行导出可消除该竞态；只影响构建耗时，产物与并发导出一致。
    staticGenerationMaxConcurrency: 1,
  },
};

export default nextConfig;
