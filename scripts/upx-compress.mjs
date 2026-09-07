/**
 * UPX 压缩：压缩主 exe 并重建 NSIS 安装包
 * UPX 路径: D:\soft\upx-5.0.2-win64\upx.exe
 */
import { execSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const tauriDir = resolve(root, 'src-tauri');
const releaseDir = resolve(tauriDir, 'target', 'release');

const UPX = 'D:\\soft\\upx-5.0.2-win64\\upx.exe';
const MAKENSIS = resolve(process.env.LOCALAPPDATA ?? '', 'tauri', 'NSIS', 'makensis.exe');

const mainExe = resolve(releaseDir, 'soloup-app.exe');
const installerOut = resolve(releaseDir, 'bundle', 'nsis', 'Soloup_0.1.0_x64-setup.exe');
const nsiScript = resolve(releaseDir, 'nsis', 'x64', 'installer.nsi');

if (!existsSync(UPX)) {
  console.error('[upx] UPX not found at', UPX);
  process.exit(1);
}

console.log('[upx] 压缩主程序...');
execSync(`"${UPX}" --best "${mainExe}"`, { stdio: 'inherit' });

if (existsSync(MAKENSIS) && existsSync(nsiScript)) {
  console.log('[upx] 重建 NSIS 安装包...');
  // 修改 nsi 脚本中的 OUTFILE 路径，然后还原
  const { readFileSync, writeFileSync } = await import('node:fs');
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
}
