// 将 backend release 二进制复制到 src-tauri/resources，带 Tauri externalBin
// 要求的目标三元组后缀。跨平台可用（不依赖 shell copy 语法）。
import { execFileSync } from 'node:child_process';
import { copyFileSync, cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));

// 目标三元组：优先取环境变量（交叉编译场景），否则用当前 rustc host
let triple = process.env.SOLOUP_TARGET_TRIPLE;
if (!triple) {
  triple = execFileSync('rustc', ['-vV'], { encoding: 'utf8' })
    .split('\n')
    .find((l) => l.startsWith('host: '))
    ?.slice('host: '.length)
    .trim();
}
if (!triple) {
  console.error('[copy-sidecars] 无法确定目标三元组（rustc -vV 失败）');
  process.exit(1);
}

const suffix = process.platform === 'win32' ? '.exe' : '';
const pairs = [
  ['soloup-server', 'soloup-server'],
  ['soloup-mcp', 'soloup-mcp'],
];

const resDir = join(root, 'src-tauri', 'resources');
mkdirSync(resDir, { recursive: true });

for (const [bin, name] of pairs) {
  const src = join(root, 'backend', 'target', 'release', `${bin}${suffix}`);
  const dest = join(resDir, `${name}-${triple}${suffix}`);
  copyFileSync(src, dest);
  console.log(`[copy-sidecars] ${src} -> ${dest}`);
}

// 前端静态产物：打包后的桌面版由 sidecar 直接托管网页，
// 这样同一台机器的「设置 → 局域网服务」才能真的把页面发给手机。
const webSrc = join(root, 'apps', 'web', 'out');
const webDest = join(resDir, 'web');
if (existsSync(webSrc)) {
  rmSync(webDest, { recursive: true, force: true });
  cpSync(webSrc, webDest, { recursive: true });
  console.log(`[copy-sidecars] ${webSrc} -> ${webDest}`);
} else {
  console.warn(
    `[copy-sidecars] 未找到前端产物 ${webSrc}，跳过（请先执行 pnpm --filter @soloup/web build）`,
  );
}

console.log(`[copy-sidecars] 目标三元组: ${triple}`);
