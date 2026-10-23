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

  // ★ 重要：makensis 重建后安装包字节已经变了，cargo tauri build 阶段生成的
  // `.sig` 因此失效。这里必须重签，否则应用内更新会在验签阶段直接失败。
  // 注意 `signer sign` 用的是 TAURI_PRIVATE_KEY / TAURI_PRIVATE_KEY_PATH，
  // 与打包器读的 TAURI_SIGNING_PRIVATE_KEY 不是同一组变量，所以这里显式传参。
  if (conf.bundle?.createUpdaterArtifacts) {
    const keyEnv = process.env.TAURI_SIGNING_PRIVATE_KEY;
    if (keyEnv) {
      console.log('[upx] 重新签名安装包（NSIS 已重建，原签名失效）...');
      const flag = existsSync(keyEnv) ? '-f' : '-k';
      const pw = process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD;
      const pwArg = pw ? ` -p "${pw}"` : '';
      execSync(`cargo tauri signer sign ${flag} "${keyEnv}"${pwArg} "${installerOut}"`, {
        cwd: tauriDir,
        stdio: 'inherit',
      });
    } else {
      console.warn('[upx] 未设置 TAURI_SIGNING_PRIVATE_KEY，安装包签名未更新');
      console.warn('[upx] 本地自用无碍；对外发布请走 CI（CI 不跑 UPX）');
    }
  }
} else {
  console.log('[upx] makensis 或 nsi 脚本未找到，跳过安装包重建');
  if (!existsSync(MAKENSIS)) console.log('[upx]   缺少:', MAKENSIS);
  if (!existsSync(nsiScript)) console.log('[upx]   缺少:', nsiScript);
}
