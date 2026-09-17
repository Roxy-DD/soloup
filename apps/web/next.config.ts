import path from 'node:path';
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'export',
  // 家目录里可能躺着一个无关的 package-lock.json，Next 据此会把用户主目录
  // （C:\Users\<user>）推断成 workspace root，导致 dev 时监视整个用户目录。
  // 显式钉回仓库根，避免这个副作用。
  outputFileTracingRoot: path.resolve(process.cwd(), '../..'),
};

export default nextConfig;
