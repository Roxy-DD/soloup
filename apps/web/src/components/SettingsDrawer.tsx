'use client';
/* ================= 设置抽屉：角色 / 生命进度轴 / 记录 / 外观与动效 / MCP 服务 / 数据 ================= */
import React, { useEffect, useState, useCallback } from 'react';
import type { AppState } from '@/lib/types';
import { Drawer } from './Drawer';
import { checkMcpHealth, getMcpToolCount, rpc } from '@/lib/api';

export function SettingsDrawer({ open, onClose, state, dispatch, toast }: {
  open: boolean;
  onClose: () => void;
  state: AppState;
  dispatch: (a: { type: 'SET_SETTINGS'; patch: Partial<AppState['settings']> } | { type: 'HYDRATE'; state: AppState }) => void;
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

  const refreshMcp = useCallback(async () => {
    const [healthy, count] = await Promise.all([checkMcpHealth(), getMcpToolCount()]);
    setMcpRunning(healthy);
    setMcpToolCount(count);
  }, []);

  // 每次打开时同步当前设置
  useEffect(() => {
    if (open) {
      setNick(s.nickname); setAvatar(s.avatar); setBirth(s.birth);
      setLifeExp(s.lifeExp); setRemind(s.remind);
      setLockHistory(s.lockHistory); setMotion(s.motion);
      refreshMcp();
    }
  }, [open]);

  const save = () => {
    if (!nick.trim()) { toast('昵称不能为空'); return; }
    if (!birth) { toast('请选择出生日期'); return; }
    dispatch({ type: 'SET_SETTINGS', patch: { nickname: nick.trim(), avatar: avatar.trim() || '人', birth, lifeExp, remind: remind || '20:00', lockHistory, motion } });
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
            if (!f) { resolve(); return; }
            const reader = new FileReader();
            reader.onload = () => { doImport(reader.result as string); resolve(); };
            reader.readAsText(f);
          };
          input.click();
        });
        return;
      }
      if (content) doImport(content);
    } catch {
      toast('导入失败，请检查文件格式');
    }
  };

  const doImport = async (raw: string) => {
    try {
      const data = JSON.parse(raw);
      const required = ['attributes', 'skills', 'links', 'records', 'recordSkills', 'projects', 'achievements', 'settings'];
      const missing = required.filter((k) => !(k in data));
      if (missing.length) { toast(`文件格式错误，缺少字段：${missing.join('、')}`); return; }
      const result = await rpc<{ imported: { attributes: number; skills: number; records: number } }>('import.all', data);
      toast(`已导入：${result.imported.attributes} 属性 · ${result.imported.skills} 技能 · ${result.imported.records} 条记录`);
      onClose();
      window.location.reload();
    } catch {
      toast('导入失败，请检查文件格式');
    }
  };

  return (
    <Drawer open={open} title="设置" onClose={onClose} label="设置">
      <div className="set-section">角色</div>
      <div className="set-row">
        <span className="lbl">昵称</span>
        <span className="ctl"><input type="text" value={nick} maxLength={8} style={{ width: 110 }} onChange={(e) => setNick(e.target.value)} /></span>
      </div>
      <div className="set-row">
        <span className="lbl">头像字符<span className="sub">一个汉字或字母</span></span>
        <span className="ctl"><input type="text" value={avatar} maxLength={1} style={{ width: 52, textAlign: 'center' }} onChange={(e) => setAvatar(e.target.value)} /></span>
      </div>

      <div className="set-section">生命进度轴</div>
      <div className="set-row">
        <span className="lbl">出生日期</span>
        <span className="ctl"><input type="date" value={birth} onChange={(e) => setBirth(e.target.value)} /></span>
      </div>
      <div className="set-row">
        <span className="lbl">预期寿命<span className="sub">只作进度轴分母，可随时改</span></span>
        <span className="ctl">
          <input type="range" min={50} max={120} step={1} value={lifeExp} onChange={(e) => setLifeExp(+e.target.value)} />
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
                const { sendNotification, isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
                let granted = await isPermissionGranted();
                if (!granted) granted = (await requestPermission()) === 'granted';
                if (granted) {
                  sendNotification({ title: '测试通知', body: '如果你看到这条通知，说明通知功能正常！' });
                  toast('通知已发送');
                } else {
                  toast('通知权限被拒绝');
                }
              } catch (e) {
                console.error('[test notification]', e);
                toast('通知失败: ' + (e as Error).message);
              }
            }}
          >测试</button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">锁定历史记录<span className="sub">48 小时前的记录不可再改</span></span>
        <span className="ctl">
          <label className="switch">
            <input type="checkbox" checked={lockHistory} onChange={(e) => setLockHistory(e.target.checked)} />
            <span className="knob" />
          </label>
        </span>
      </div>

      <div className="set-section">外观与动效</div>
      <div className="set-row">
        <span className="lbl">界面动效<span className="sub">关闭后所有动画立即完成（无障碍）</span></span>
        <span className="ctl">
          <label className="switch">
            <input type="checkbox" checked={motion} onChange={(e) => setMotion(e.target.checked)} />
            <span className="knob" />
          </label>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">主题</span>
        <span className="ctl"><span className="tag">明亮街机</span></span>
      </div>

      <div className="set-section">MCP 服务</div>
      <div className="set-row">
        <span className="lbl">状态</span>
        <span className="ctl">
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
            <span style={{
              width: 8, height: 8, borderRadius: '50%',
              background: mcpRunning ? 'var(--green, #4ade80)' : 'var(--red, #f87171)',
              display: 'inline-block',
            }} />
            {mcpRunning ? '运行中' : '未启动'}
          </span>
          <button
            className="btn-ghost"
            style={{ marginLeft: 8, fontSize: 11, padding: '2px 6px' }}
            onClick={refreshMcp}
          >刷新</button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">连接地址</span>
        <span className="ctl">
          <code style={{ fontSize: 12, background: 'var(--bg-2, #1a1a2e)', color: '#e0e0e0', padding: '4px 8px', borderRadius: 4 }}>
            http://localhost:8788/mcp
          </code>
          <button
            className="btn-ghost"
            style={{ marginLeft: 6, fontSize: 11, padding: '2px 6px' }}
            onClick={() => {
              navigator.clipboard.writeText('http://localhost:8788/mcp');
              toast('已复制连接地址');
            }}
          >复制</button>
        </span>
      </div>
      <div className="set-row">
        <span className="lbl">已注册工具</span>
        <span className="ctl">{mcpRunning ? `${mcpToolCount} 个` : '—'}</span>
      </div>
      <div className="set-row">
        <span className="lbl">AI 客户端配置<span className="sub">添加到 Claude Desktop 等工具的 MCP 设置</span></span>
        <span className="ctl">
          <code style={{ fontSize: 11, background: 'var(--panel)', padding: '2px 6px', borderRadius: 4, cursor: 'pointer' }}
            onClick={() => {
              navigator.clipboard.writeText(JSON.stringify({
                mcpServers: {
                  soloup: { url: 'http://localhost:8788/mcp' }
                }
              }, null, 2));
              toast('已复制 MCP 配置');
            }}>
            点击复制配置
          </code>
        </span>
      </div>

      <div className="set-section">数据</div>
      <div className="form-ops" style={{ marginTop: 8 }}>
        <button className="btn-ghost" onClick={exportData}>导出 JSON</button>
        <button className="btn-ghost" onClick={importData}>导入 JSON</button>
      </div>
      <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 10 }}>
        数据存储在本地 SQLite 数据库（~/.soloup/soloup.db）；导出 JSON 可用于备份或迁移。
      </p>
      <div className="form-ops">
        <button className="btn-primary" onClick={save}>保存设置</button>
        <button className="btn-ghost" onClick={onClose}>取消</button>
      </div>
    </Drawer>
  );
}
