'use client';
/* ================= 人生 RPG 面板 · 主页面：导航 + 五页 Tab + 抽屉编排 + 成就引擎 ================= */
import React, { useEffect, useState } from 'react';
import { StoreProvider, useStore } from '@/lib/store';
import { ToastProvider, useToast } from '@/components/Toast';
import { startReminder } from '@/lib/reminder';
import { NavIcon } from '@/components/icons';
import { DashboardTab } from '@/components/DashboardTab';
import { SkillsTab } from '@/components/SkillsTab';
import { CheckinTab } from '@/components/CheckinTab';
import { ProjectsTab } from '@/components/ProjectsTab';
import { AchievementsTab } from '@/components/AchievementsTab';
import { AttrDrawer } from '@/components/AttrDrawer';
import { SettingsDrawer } from '@/components/SettingsDrawer';

type TabId = 'dashboard' | 'skills' | 'log' | 'projects' | 'achievements';
const TABS: { id: TabId; label: string }[] = [
  { id: 'dashboard', label: '面板' },
  { id: 'skills', label: '技能' },
  { id: 'log', label: '记录' },
  { id: 'projects', label: '项目' },
  { id: 'achievements', label: '成就' },
];

function Shell() {
  const { state, dispatch } = useStore();
  const toast = useToast();
  const [tab, setTab] = useState<TabId>('dashboard');
  const [attrMgrOpen, setAttrMgrOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

  // 动效总开关：设置关闭时立即完成所有动画（无障碍）
  useEffect(() => {
    document.documentElement.classList.toggle('no-motion', !state.settings.motion);
  }, [state.settings.motion]);

  // 禁用右键菜单（Tauri 桌面应用不应暴露浏览器右键）
  useEffect(() => {
    const handler = (e: MouseEvent) => e.preventDefault();
    document.addEventListener('contextmenu', handler);
    return () => document.removeEventListener('contextmenu', handler);
  }, []);

  // 每日打卡提醒
  useEffect(() => {
    const cleanup = startReminder({
      getRemindTime: () => state.settings.remind,
      toast,
    });
    return cleanup;
  }, [state.settings.remind]);

  const go = (t: TabId) => {
    setTab(t);
    window.scrollTo({ top: 0 });
  };

  const nav = (id: string, label: string) => (
    <button key={id} className={tab === id ? 'active' : ''} onClick={() => go(id as TabId)} aria-label={label}>
      <NavIcon name={id} />
      {label}
    </button>
  );

  return (
    <div className="app">
      <aside className="sidebar" aria-label="主导航">
        <div className="logo">人生RPG</div>
        {TABS.map((t) => nav(t.id, t.label))}
      </aside>

      <main>
        {tab === 'dashboard' && (
          <DashboardTab
            state={state}
            onCheckin={() => go('log')}
            onOpenAttrMgr={() => setAttrMgrOpen(true)}
            onOpenSettings={() => setSettingsOpen(true)}
            onOpenSkill={() => go('skills')}
          />
        )}
        {tab === 'skills' && <SkillsTab state={state} dispatch={dispatch} toast={toast} />}
        {tab === 'log' && <CheckinTab state={state} dispatch={dispatch} />}
        {tab === 'projects' && <ProjectsTab state={state} dispatch={dispatch} />}
        {tab === 'achievements' && <AchievementsTab state={state} />}
      </main>

      <nav className="tabbar" aria-label="底部导航">
        {TABS.map((t) => nav(t.id, t.label))}
      </nav>

      <AttrDrawer open={attrMgrOpen} onClose={() => setAttrMgrOpen(false)} state={state} dispatch={dispatch} toast={toast} />
      <SettingsDrawer open={settingsOpen} onClose={() => setSettingsOpen(false)} state={state} dispatch={dispatch} toast={toast} />
    </div>
  );
}

export default function Home() {
  return (
    <StoreProvider>
      <ToastProvider>
        <Shell />
      </ToastProvider>
    </StoreProvider>
  );
}
