/**
 * UPX 压缩：压缩主 exe 并重建 NSIS 安装包
 *
 * 默认从 D:\soft\upx-5.0.2-win64\upx.exe 取 UPX，可用环境变量 UPX_BIN 覆盖。
 * 安装包名不再写死版本号：优先复用 bundle/nsis 下已有的 *-setup.exe，
 * 否则按 tauri.conf.json 的 version 推算 Soloup_<version>_x64-setup.exe。
 */
import { execSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const tauriDir = resolve(root, 'src-tauri');
const releaseDir = resolve(tauriDir, 'target', 'release');
const nsisDir = resolve(releaseDir, 'bundle', 'nsis');

const UPX = process.env.UPX_BIN ?? 'D:\\soft\\upx-5.0.2-win64\\upx.exe';
const MAKENSIS = resolve(process.env.LOCALAPPDATA ?? '', 'tauri', 'NSIS', 'makensis.exe');

const mainExe = resolve(releaseDir, 'soloup-app.exe');
const nsiScript = resolve(releaseDir, 'nsis', 'x64', 'installer.nsi');

// 版本号单一来源：tauri.conf.json
const conf = JSON.parse(readFileSync(resolve(tauriDir, 'tauri.conf.json'), 'utf8'));
const productName = conf.productName ?? 'Soloup';
const version = conf.version ?? '0.0.0';

/** 安装包输出路径：优先取目录里已存在的产物，否则按 productName_version_x64-setup.exe 推算 */
function resolveInstallerOut() {
  if (existsSync(nsisDir)) {
    const found = readdirSync(nsisDir).filter((f) => f.endsWith('-setup.exe'));
    const exact = found.find((f) => f.includes(`_${version}_`));
    if (exact) return join(nsisDir, exact);
    if (found.length === 1) return join(nsisDir, found[0]);
  }
  return join(nsisDir, `${productName}_${version}_x64-setup.exe`);
}

const installerOut = resolveInstallerOut();

if (!existsSync(mainExe)) {
  console.error('[upx] 未找到主程序', mainExe, '——请先执行 cargo tauri build');
  process.exit(1);
}

if (!existsSync(UPX)) {
  // 安装包在此步骤之前已由 cargo tauri build 产出，UPX 缺失不应视为构建失败
  console.warn('[upx] 未找到 UPX（', UPX, '），跳过压缩；安装包保持未压缩状态');
  console.warn('[upx] 可用环境变量 UPX_BIN 指定路径');
  process.exit(0);
}

console.log('[upx] 压缩主程序...');
execSync(`"${UPX}" --best "${mainExe}"`, { stdio: 'inherit' });

if (existsSync(MAKENSIS) && existsSync(nsiScript)) {
  console.log('[upx] 重建 NSIS 安装包 ->', installerOut);
  // 修改 nsi 脚本中的 OUTFILE 路径，然后还原
  const orig = readFileSync(nsiScript, 'utf8');
  const patched = orig.replace(
    '!define OUTFILE "nsis-output.exe"',
    `!define OUTFILE "${installerOut.replace(/\\/g, '\\\\')}"`,
  );
  writeFileSync(nsiScript, patched);
  try {
    execSync(`"${MAKENSIS}" -V1 "${nsiScript}"`, { stdio: 'inherit' });
  } finally {
    writeFileSync(nsiScript, orig);
  }
  console.log('[upx] 安装包已更新:', installerOut);
} else {
  console.log('[upx] makensis 或 nsi 脚本未找到，跳过安装包重建');
  if (!existsSync(MAKENSIS)) console.log('[upx]   缺少:', MAKENSIS);
  if (!existsSync(nsiScript)) console.log('[upx]   缺少:', nsiScript);
}
