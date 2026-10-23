'use client';
/* ================= 设置抽屉：角色 / 生命进度轴 / 记录 / 外观与动效 / MCP 服务 / 数据 ================= */
import React, { useEffect, useRef, useState, useCallback } from 'react';
import type { AppState } from '@/lib/types';
import { Drawer } from './Drawer';
import { checkMcpHealth, getMcpToolCount, rpc } from '@/lib/api';
import { getAutostart, isDesktop, setAutostart } from '@/lib/autostart';
import {
  canUpdate,
  checkUpdate,
  getAppVersion,
  installUpdate,
  onUpdateProgress,
  type UpdateInfo,
} from '@/lib/update';

/**
 * 复制文本。
 *
 * 手机通过局域网 http:// 打开时页面不是安全上下文（secure context），
 * navigator.clipboard 直接不可用，所以这里备一条 execCommand 降级路径；
 * 两条都失败就返回 false，由调用方提示用户长按手动复制。
 */
async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // 落到下面的降级方案
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.top = '-1000px';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, ta.value.length);
    const ok = document.execCommand('copy');
    ta.remove();
    return ok;
  } catch {
    return false;
  }
}

/** soloup-server 的局域网状态（lan.status / lan.set 的返回体） */
type LanStatus = {
  enabled: boolean;
  port: number;
  ip: string | null;
  url: string | null;
  webReady: boolean;
};

export function SettingsDrawer({
  open,
  onClose,
  state,
  dispatch,
  toast,
}: {
  open: boolean;
  onClose: () => void;
  state: AppState;
  dispatch: (
    a:
      | { type: 'SET_SETTINGS'; patch: Partial<AppState['settings']> }
      | { type: 'HYDRATE'; state: AppState },
  ) => void;
  toast: (m: string) => void;
}) {
  const s = state.settings;
  const [nick, setNick] = useState(s.nickname);
  const [avatar, setAvatar] = useState(s.avatar);
  const [birth, setBirth] = useState(s.birth);
  const [lifeExp, setLifeExp] = useState(s.lifeExp);
  const [remind, setRemind] = useState(s.remind);
  const [lockHistory, setLockHistory] = useState(s.lockHistory);
  const [motion, setMotion] = useState(s.motion);

  const [mcpRunning, setMcpRunning] = useState(false);
  const [mcpToolCount, setMcpToolCount] = useState(0);

  // 局域网服务：开关存在后端 settings 里，这里只做展示与切换
  const [lan, setLan] = useState<LanStatus | null>(null);
  const [lanBusy, setLanBusy] = useState(false);
  // 访问地址的二维码（后端 qrcode 生成的 SVG 片段），地址变了就重新拉
  const [qrSvg, setQrSvg] = useState<string | null>(null);

  // 开机自启动：状态直接读系统（Windows 注册表的 Run 项），不落后端数据库。
  // null = 当前环境没有这个能力（浏览器 / 手机局域网页面）。
  const [autoStart, setAutoStart] = useState<boolean | null>(null);
  const [autoStartBusy, setAutoStartBusy] = useState(false);

  const refreshMcp = useCallback(async () => {
    const [healthy, count] = await Promise.all([checkMcpHealth(), getMcpToolCount()]);
    setMcpRunning(healthy);
    setMcpToolCount(count);
  }, []);

  const refreshLan = useCallback(async () => {
    try {
      const st = await rpc<LanStatus>('lan.status');
      setLan(st);
      // 只有拿到地址才需要二维码；否则清掉，避免留一个过期的码
      if (st.url) {
        const q = await rpc<{ url: string | null; svg: string | null }>('lan.qr');
        setQrSvg(q.svg);
      } else {
        setQrSvg(null);
      }
    } catch {
      setLan(null);
      setQrSvg(null);
    }
  }, []);

  const toggleLan = useCallback(
    async (next: boolean) => {
      setLanBusy(true);
      try {
        await rpc<LanStatus>('lan.set', { enabled: next });
        // 开关会改变 url 的有效性，重新拉一次状态与二维码
        await refreshLan();
        toast(next ? '局域网服务已开启' : '局域网服务已关闭');
      } catch (e) {
        toast('操作失败：' + (e as Error).message);
      } finally {
        setLanBusy(false);
      }
    },
    [toast, refreshLan],
  );

  const refreshAutoStart = useCallback(async () => {
    setAutoStart(await getAutostart());
  }, []);

  const toggleAutoStart = useCallback(
    async (next: boolean) => {
      setAutoStartBusy(true);
      const ok = await setAutostart(next);
      if (ok) {
        setAutoStart(next);
        toast(next ? '已开启开机自启动' : '已关闭开机自启动');
      } else {
        // 写失败就回读系统的真实状态，别让开关停在用户点的哪一侧
        setAutoStart(await getAutostart());
        toast('设置失败，可能被系统策略拦住了');
      }
      setAutoStartBusy(false);
    },
    [toast],
  );

  /* ---------------- 应用内更新 ----------------
   * 只走 Rust 侧命令（见 lib/update.ts）。检查失败不当异常抛出，而是落到
   * 'error' 状态里显示，避免断网时面板一片安静、用户以为按钮坏了。
   */
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [updPhase, setUpdPhase] = useState<
    'idle' | 'checking' | 'latest' | 'available' | 'installing' | 'error'
  >('idle');
  const [updInfo, setUpdInfo] = useState<UpdateInfo | null>(null);
  const [updError, setUpdError] = useState<string | null>(null);
  const [updProgress, setUpdProgress] = useState<{
    downloaded: number;
    total: number | null;
  } | null>(null);

  const runCheck = useCallback(
    async (announce: boolean) => {
      if (!canUpdate()) {
        if (announce) toast('仅桌面版支持应用内更新');
        return;
      }
      setUpdPhase('checking');
      setUpdError(null);
      const res = await checkUpdate();
      if (!res.ok) {
        setUpdPhase('error');
        setUpdError(res.error);
        if (announce) toast('检查更新失败：' + res.error);
        return;
      }
      if (res.info) {
        setUpdInfo(res.info);
        setUpdPhase('available');
        if (announce) toast(`发现新版本 v${res.info.version}`);
      } else {
        setUpdPhase('latest');
        if (announce) toast('已经是最新版本');
      }
    },
    [toast],
  );

  const runInstall = useCallback(async () => {
    setUpdPhase('installing');
    setUpdProgress({ downloaded: 0, total: null });
    setUpdError(null);
    try {
      // 成功路径下进程会被安装器接管，这里不会返回
      await installUpdate();
    } catch (e) {
      setUpdPhase('error');
      setUpdError((e as Error).message || String(e));
    }
  }, []);

  // 安装阶段订阅进度事件（phase 变化才订阅，避免常驻监听）
  useEffect(() => {
    if (updPhase !== 'installing') return;
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void onUpdateProgress((p) => setUpdProgress({ downloaded: p.downloaded, total: p.total })).then(
      (fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      },
    );
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [updPhase]);

  // 启动后静默检查一次：发现新版本只在设置页里显示，不弹窗打扰
  const autoChecked = useRef(false);
  const runCheckRef = useRef(runCheck);
  useEffect(() => {
    runCheckRef.current = runCheck;
  }, [runCheck]);
  useEffect(() => {
    if (autoChecked.current || !canUpdate()) return;
    autoChecked.current = true;
    void getAppVersion().then(setAppVersion);
    const timer = setTimeout(() => void runCheckRef.current(false), 5000);
    return () => clearTimeout(timer);
  }, []);

  // 每次打开时同步当前设置
  useEffect(() => {
    if (open) {
      setNick(s.nickname);
      setAvatar(s.avatar);
      setBirth(s.birth);
      setLifeExp(s.lifeExp);
      setRemind(s.remind);
      setLockHistory(s.lockHistory);
      setMotion(s.motion);
      refreshMcp();
      refreshLan();
      refreshAutoStart();
    }
  }, [open]);

  const save = () => {
    if (!nick.trim()) {
      toast('昵称不能为空');
      return;
    }
    if (!birth) {
      toast('请选择出生日期');
      return;
    }
    dispatch({
      type: 'SET_SETTINGS',
      patch: {
        nickname: nick.trim(),
        avatar: avatar.trim() || '人',
        birth,
        lifeExp,
        remind: remind || '20:00',
        lockHistory,
        motion,
      },
    });
    toast('设置已保存');
    onClose();
  };

  const exportData = async () => {
    try {
      const data = await rpc('export.all');
      const json = JSON.stringify(data, null, 2);
      try {
        const { save } = await import('@tauri-apps/plugin-dialog');
        const { writeTextFile } = await import('@tauri-apps/plugin-fs');
        const filePath = await save({
          filters: [{ name: 'JSON', extensions: ['json'] }],
          defaultPath: `soloup-export-${new Date().toISOString().slice(0, 10)}.json`,
        });
        if (filePath) {
          await writeTextFile(filePath, json);
          toast('已导出数据库备份');
        }
      } catch {
        const blob = new Blob([json], { type: 'application/json' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = `soloup-export-${new Date().toISOString().slice(0, 10)}.json`;
        document.body.appendChild(a);
        a.click();
        a.remove();
        toast('已导出数据库备份');
      }
    } catch (e) {
      console.error('[export] error:', e);
      toast('导出失败，请检查后端连接');
    }
  };

  const importData = async () => {
    try {
      let content: string | undefined;
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const { readTextFile } = await import('@tauri-apps/plugin-fs');
        const filePath = await open({
          filters: [{ name: 'JSON', extensions: ['json'] }],
          multiple: false,
        });
        if (!filePath) return;
        content = await readTextFile(filePath as string);
      } catch {
        await new Promise<void>((resolve) => {
          const input = document.createElement('input');
          input.type = 'file';
          input.accept = '.json';
          input.onchange = () => {
            const f = input.files?.[0];
            if (!f) {
              resolve();
              return;
            }
            const reader = new FileReader();
            reader.onload = () => {
              doImport(reader.result as string);
              resolve();
            };
            reader.readAsText(f);
          };
          input.click();
        });
        return;
      }
      if (content) doImport(content);
    } catch (error) {
      console.error('[import] error:', error);
      toast(error instanceof Error ? `导入失败：${error.message}` : '导入失败，请检查文件格式');
    }
  };

  const doImport = async (raw: string) => {
    try {
      const data = JSON.parse(raw);
      const required = [
        'attributes',
        'skills',
        'links',
        'records',
        'recordSkills',
        'projects',
        'achievements',
        'settings',
      ];
      const missing = required.filter((k) => !(k in data));
      if (missing.length) {
        toast(`文件格式错误，缺少字段：${missing.join('、')}`);
        return;
      }
      const result = await rpc<{
        imported: { attributes: number; skills: number; records: number };
      }>('import.all', data);
      toast(
        `已导入：${result.imported.attributes} 属性 · ${result.imported.skills} 技能 · ${result.imported.records} 条记录`,
      );
      onClose();
      window.location.reload();
    } catch (error) {
      console.error('[import] error:', error);
      toast(error instanceof Error ? `导入失败：${error.message}` : '导入失败，请检查文件格式');
    }
  };

  // 提示文案分三种情况说。别把「读不到」误报成「不支持」—— 在桌面版里
  // 显示「仅桌面版可设置」会是一句让人摸不着头脑的假话。
  let autoStartHint: string;
  if (!isDesktop()) {
    autoStartHint = '仅桌面版可设置：网页（含手机通过局域网打开的页面）没有修改系统启动项的权限。';
  } else if (autoStart === null) {
    autoStartHint = '没能读到系统的启动项设置，关掉重开这个面板再试一次。';
  } else {
    autoStartHint =
      '开关即时生效，不需要点「保存设置」。开启后开机只在托盘后台运行、不弹窗；想看面板时点右下角托盘图标即可。';
  }

  return (
    <Drawer open={open} title="设置" onClose={onClose} label="设置">
      <div className="set-section">角色</div>
      <div className="set-row">
        <span className="lbl">昵称</span>
        <span className="ctl">
          <input
            type="text"
            value={nick}
            maxLength={8}
            style={{ width: 110 }}
            onChange={(e) => setNick(e.target.value)}
          />
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">
          头像字符<span className="sub">一个汉字或字母</span>
        </span>
        <span className="ctl">
          <input
            type="text"
            value={avatar}
            maxLength={1}
            style={{ width: 52, textAlign: 'center' }}
            onChange={(e) => setAvatar(e.target.value)}
          />
        </span>
      </div>

      <div className="set-section">生命进度轴</div>
      <div className="set-row">
        <span className="lbl">出生日期</span>
        <span className="ctl">
          <input type="date" value={birth} onChange={(e) => setBirth(e.target.value)} />
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">
          预期寿命<span className="sub">只作进度轴分母，可随时改</span>
        </span>
        <span className="ctl">
          <input
            type="range"
            min={50}
            max={120}
            step={1}
            value={lifeExp}
            onChange={(e) => setLifeExp(+e.target.value)}
          />
          <span className="range-val">{lifeExp}</span>
        </span>
      </div>

      <div className="set-section">记录</div>
      <div className="set-row">
        <span className="lbl">每日打卡提醒</span>
        <span className="ctl">
          <input type="time" value={remind} onChange={(e) => setRemind(e.target.value)} />
          <button
            className="btn-ghost"
            style={{ marginLeft: 6, fontSize: 11, padding: '2px 6px' }}
            onClick={async () => {
              try {
                const { sendNotification, isPermissionGranted, requestPermission } =
                  await import('@tauri-apps/plugin-notification');
                let granted = await isPermissionGranted();
                if (!granted) granted = (await requestPermission()) === 'granted';
                if (granted) {
                  sendNotification({
                    title: '测试通知',
                    body: '如果你看到这条通知，说明通知功能正常！',
                  });
                  toast('通知已发送');
                } else {
                  toast('通知权限被拒绝');
                }
              } catch (e) {
                console.error('[test notification]', e);
                toast('通知失败: ' + (e as Error).message);
              }
            }}
          >
            测试
          </button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">
          锁定历史记录<span className="sub">48 小时前的记录不可再改</span>
        </span>
        <span className="ctl">
          <label className="switch">
            <input
              type="checkbox"
              checked={lockHistory}
              onChange={(e) => setLockHistory(e.target.checked)}
            />
            <span className="knob" />
          </label>
        </span>
      </div>

      <div className="set-section">启动</div>
      <div className="set-row">
        <span className="lbl">
          开机自启动<span className="sub">登录 Windows 后自动在托盘后台运行</span>
        </span>
        <span className="ctl">
          <label className="switch">
            <input
              type="checkbox"
              checked={autoStart ?? false}
              disabled={autoStart === null || autoStartBusy}
              onChange={(e) => toggleAutoStart(e.target.checked)}
            />
            <span className="knob" />
          </label>
        </span>
      </div>
      <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 6, lineHeight: 1.6 }}>
        {autoStartHint}
      </p>

      <div className="set-section">关于与更新</div>
      <div className="set-row">
        <span className="lbl">
          Soloup
          <span className="sub">
            {appVersion ? `当前版本 v${appVersion}` : '版本信息仅桌面版可见'}
          </span>
        </span>
        <span className="ctl">
          <button
            className="btn-ghost"
            style={{ fontSize: 11, padding: '2px 8px' }}
            disabled={!canUpdate() || updPhase === 'checking' || updPhase === 'installing'}
            onClick={() => void runCheck(true)}
          >
            {updPhase === 'checking' ? '检查中…' : '检查更新'}
          </button>
        </span>
      </div>

      {updPhase === 'available' && updInfo && (
        <div className="set-row">
          <span className="lbl">
            发现 v{updInfo.version}
            <span className="sub">更新会保留全部本地数据与设置</span>
          </span>
          <span className="ctl">
            <button
              className="btn-primary"
              style={{ fontSize: 11, padding: '3px 10px' }}
              onClick={() => void runInstall()}
            >
              立即更新
            </button>
          </span>
        </div>
      )}

      {updPhase === 'installing' && (
        <div style={{ margin: '8px 0 4px' }}>
          <div
            style={{
              height: 4,
              borderRadius: 2,
              background: 'var(--bg-2, #1a1a2e)',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                height: '100%',
                width:
                  updProgress?.total && updProgress.total > 0
                    ? `${Math.min(100, Math.round((updProgress.downloaded / updProgress.total) * 100))}%`
                    : '25%',
                background: 'var(--accent, #a78bfa)',
                transition: 'width .2s linear',
              }}
            />
          </div>
          <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 5, lineHeight: 1.6 }}>
            {updProgress?.total && updProgress.total > 0
              ? `已下载 ${(updProgress.downloaded / 1048576).toFixed(1)} / ${(
                  updProgress.total / 1048576
                ).toFixed(1)} MB`
              : '正在下载安装包…'}
            <br />
            下载完成后应用会自动关闭并重启，期间无需操作。
          </p>
        </div>
      )}

      {updPhase === 'latest' && (
        <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 6 }}>已是最新版本。</p>
      )}

      {updPhase === 'error' && (
        <p style={{ fontSize: 11, color: 'var(--red, #f87171)', marginTop: 6, lineHeight: 1.6 }}>
          检查更新失败：{updError}
          <br />
          可以在 GitHub Releases 页面手动下载安装包。
        </p>
      )}

      <div className="set-section">外观与动效</div>
      <div className="set-row">
        <span className="lbl">
          界面动效<span className="sub">关闭后所有动画立即完成（无障碍）</span>
        </span>
        <span className="ctl">
          <label className="switch">
            <input type="checkbox" checked={motion} onChange={(e) => setMotion(e.target.checked)} />
            <span className="knob" />
          </label>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">主题</span>
        <span className="ctl">
          <span className="tag">明亮街机</span>
        </span>
      </div>

      <div className="set-section">MCP 服务</div>
      <div className="set-row">
        <span className="lbl">状态</span>
        <span className="ctl">
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: '50%',
                background: mcpRunning ? 'var(--green, #4ade80)' : 'var(--red, #f87171)',
                display: 'inline-block',
              }}
            />
            {mcpRunning ? '运行中' : '未启动'}
          </span>
          <button
            className="btn-ghost"
            style={{ marginLeft: 8, fontSize: 11, padding: '2px 6px' }}
            onClick={refreshMcp}
          >
            刷新
          </button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">连接地址</span>
        <span className="ctl">
          <code
            style={{
              fontSize: 12,
              background: 'var(--bg-2, #1a1a2e)',
              color: '#e0e0e0',
              padding: '4px 8px',
              borderRadius: 4,
            }}
          >
            http://localhost:8788/mcp
          </code>
          <button
            className="btn-ghost"
            style={{ marginLeft: 6, fontSize: 11, padding: '2px 6px' }}
            onClick={async () => {
              const ok = await copyText('http://localhost:8788/mcp');
              toast(ok ? '已复制连接地址' : '复制失败，请手动复制');
            }}
          >
            复制
          </button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">已注册工具</span>
        <span className="ctl">{mcpRunning ? `${mcpToolCount} 个` : '—'}</span>
      </div>
      <div className="set-row">
        <span className="lbl">
          AI 客户端配置<span className="sub">添加到 Claude Desktop 等工具的 MCP 设置</span>
        </span>
        <span className="ctl">
          <code
            style={{
              fontSize: 11,
              background: 'var(--panel)',
              padding: '2px 6px',
              borderRadius: 4,
              cursor: 'pointer',
            }}
            onClick={async () => {
              const ok = await copyText(
                JSON.stringify(
                  {
                    mcpServers: {
                      soloup: { url: 'http://localhost:8788/mcp' },
                    },
                  },
                  null,
                  2,
                ),
              );
              toast(ok ? '已复制 MCP 配置' : '复制失败，请手动复制');
            }}
          >
            点击复制配置
          </code>
        </span>
      </div>

      <div className="set-section">局域网服务</div>
      <div className="set-row">
        <span className="lbl">
          手机访问<span className="sub">同一 Wi-Fi 下可直接打开面板</span>
        </span>
        <span className="ctl">
          <label className="switch">
            <input
              type="checkbox"
              checked={lan?.enabled ?? false}
              disabled={lanBusy}
              onChange={(e) => toggleLan(e.target.checked)}
            />
            <span className="knob" />
          </label>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">访问地址</span>
        <span className="ctl">
          <code
            style={{
              fontSize: 12,
              background: 'var(--panel)',
              color: 'var(--ink)',
              padding: '4px 8px',
              borderRadius: 4,
            }}
          >
            {lan?.url ?? (lan ? '未探测到局域网地址' : '后端未连接')}
          </code>
          {lan?.url && (
            <button
              className="btn-ghost"
              style={{ marginLeft: 6, fontSize: 11, padding: '2px 6px' }}
              onClick={async () => {
                const ok = await copyText(lan.url as string);
                toast(ok ? '已复制访问地址' : '复制失败，请长按地址手动复制');
              }}
            >
              复制
            </button>
          )}
        </span>
      </div>
      {lan?.enabled && qrSvg && (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: 8,
            margin: '12px 0 2px',
          }}
        >
          {/* SVG 由后端 qrcode 生成，只含码点路径与颜色，无用户输入 */}
          <div className="lan-qr" dangerouslySetInnerHTML={{ __html: qrSvg }} />
          <span style={{ fontSize: 11, color: 'var(--text-3)' }}>手机相机扫一扫，直接打开</span>
        </div>
      )}
      {lan && !lan.webReady && (
        <p style={{ fontSize: 11, color: 'var(--red-deep)', marginTop: 6, lineHeight: 1.6 }}>
          还没找到网页产物：请先在项目根目录执行 <code>pnpm --filter @soloup/web build</code>{' '}
          生成静态页面。
        </p>
      )}
      <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 6, lineHeight: 1.6 }}>
        手机与电脑连同一个 Wi-Fi，用 Safari 打开上面的地址，点「分享 → 添加到主屏幕」，即可像 App
        一样打开。
      </p>

      <div className="set-section">数据</div>
      <div className="form-ops" style={{ marginTop: 8 }}>
        <button className="btn-ghost" onClick={exportData}>
          导出 JSON
        </button>
        <button className="btn-ghost" onClick={importData}>
          导入 JSON
        </button>
      </div>
      <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 10 }}>
        数据保存在运行 Soloup 的电脑本地。桌面安装版使用
        %APPDATA%\com.soloup.app\soloup.db；直接运行后端时使用 ~/.soloup/soloup.db。导出 JSON
        可用于备份或迁移。
      </p>
      <div className="form-ops">
        <button className="btn-primary" onClick={save}>
          保存设置
        </button>
        <button className="btn-ghost" onClick={onClose}>
          取消
        </button>
      </div>
    </Drawer>
  );
}
