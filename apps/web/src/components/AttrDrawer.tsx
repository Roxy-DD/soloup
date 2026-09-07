'use client';
/* ================= 属性管理抽屉：列表 + 新建/编辑表单 + 删除二次确认 ================= */
import React, { useEffect, useRef, useState } from 'react';
import type { AppState } from '@/lib/types';
import { attrLv, collectLeavesAll } from '@/lib/model';
import { ATTR_PALETTE } from '@/lib/seed';
import { Drawer } from './Drawer';

type Mode = { kind: 'list' } | { kind: 'add' } | { kind: 'edit'; id: string };

export function AttrDrawer({ open, onClose, state, dispatch, toast }: {
  open: boolean;
  onClose: () => void;
  state: AppState;
  dispatch: (a: { type: 'ATTR_ADD'; name: string; en: string; palette: [string, string] } | { type: 'ATTR_UPDATE'; id: string; name: string; en: string; palette: [string, string] } | { type: 'ATTR_DELETE'; id: string } | { type: 'ATTR_MOVE'; index: number; dir: -1 | 1 }) => void;
  toast: (m: string) => void;
}) {
  const [mode, setMode] = useState<Mode>({ kind: 'list' });
  const [confirmDel, setConfirmDel] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [en, setEn] = useState('');
  const [colorIdx, setColorIdx] = useState(0);
  const nameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) { setMode({ kind: 'list' }); setConfirmDel(null); }
  }, [open]);

  useEffect(() => {
    if (mode.kind !== 'list') {
      const t = setTimeout(() => nameRef.current?.focus(), 80);
      return () => clearTimeout(t);
    }
  }, [mode]);

  const startEdit = (id: string) => {
    const a = state.attrs.find((x) => x.id === id);
    if (!a) return;
    const idx = ATTR_PALETTE.findIndex(([c]) => c === a.color);
    setName(a.name); setEn(a.en); setColorIdx(idx < 0 ? 0 : idx);
    setMode({ kind: 'edit', id });
  };

  const submit = () => {
    const n = name.trim();
    const e = en.trim().toUpperCase();
    if (!n) { toast('请填写属性名称'); return; }
    if (e.length < 2) { toast('英文缩写至少 2 个字母'); return; }
    const palette = ATTR_PALETTE[colorIdx] ?? ATTR_PALETTE[0];
    if (mode.kind === 'edit') {
      dispatch({ type: 'ATTR_UPDATE', id: mode.id, name: n, en: e, palette });
      toast('已保存属性修改');
    } else {
      if (state.attrs.some((a) => a.name === n)) { toast('已有同名属性'); return; }
      dispatch({ type: 'ATTR_ADD', name: n, en: e, palette });
      toast(state.attrs.length + 1 > 8 ? '已创建 · 属性超过 8 个，雷达图已切换为排行视图' : `已创建属性「${n}」`);
    }
    setMode({ kind: 'list' });
  };

  const askDelete = (id: string) => {
    if (state.attrs.length <= 3) { toast('至少保留 3 个属性（雷达图下限）'); return; }
    setConfirmDel(id);
  };

  const doDelete = (id: string) => {
    const a = state.attrs.find((x) => x.id === id);
    const refs = collectLeavesAll(state.skills).filter((l) => l.attrs.some(([aid]) => aid === id)).length;
    dispatch({ type: 'ATTR_DELETE', id });
    setConfirmDel(null);
    toast(`已删除「${a?.name ?? ''}」${refs ? ` · ${refs} 个技能已解除关联` : ''}`);
  };

  const editing = mode.kind === 'edit' ? state.attrs.find((a) => a.id === mode.id) : null;

  return (
    <Drawer open={open} title="属性管理" onClose={onClose} label="属性管理">
      {mode.kind === 'list' && (
        <>
          <p style={{ fontSize: 12, color: 'var(--text-3)', marginBottom: 6 }}>
            3–8 个属性时显示雷达图；超过 8 个自动切换为排行条视图。删除被技能引用的属性会先解除关联。
          </p>
          {state.attrs.map((a, i) => (
            <React.Fragment key={a.id}>
              <div className="attr-row">
                <span className="sw" style={{ background: a.color }} />
                <span className="nm2">{a.name}</span>
                <span className="en2">{a.en}</span>
                <span className="lv2">LV{attrLv(a.ep)}</span>
                <span className="sp" />
                <button className="icon-btn" disabled={i === 0} onClick={() => dispatch({ type: 'ATTR_MOVE', index: i, dir: -1 })} aria-label={`上移${a.name}`}>↑</button>
                <button className="icon-btn" disabled={i === state.attrs.length - 1} onClick={() => dispatch({ type: 'ATTR_MOVE', index: i, dir: 1 })} aria-label={`下移${a.name}`}>↓</button>
                <button className="icon-btn" onClick={() => startEdit(a.id)} aria-label={`编辑${a.name}`}>✎</button>
                <button className="icon-btn del" onClick={() => askDelete(a.id)} aria-label={`删除${a.name}`}>删</button>
              </div>
              {confirmDel === a.id && (() => {
                const refs = collectLeavesAll(state.skills).filter((l) => l.attrs.some(([aid]) => aid === a.id)).length;
                return (
                  <div className="warn-box" style={{ margin: '4px 0 8px 0' }}>
                    确定删除「{a.name}」？{refs ? `${refs} 个技能将解除关联。` : ''}
                    <div className="form-ops">
                      <button className="btn-primary" onClick={() => doDelete(a.id)}>确认删除</button>
                      <button className="btn-ghost" onClick={() => setConfirmDel(null)}>再想想</button>
                    </div>
                  </div>
                );
              })()}
            </React.Fragment>
          ))}
          <div className="form-ops">
            <button className="btn-primary" onClick={() => { setName(''); setEn(''); setColorIdx(0); setMode({ kind: 'add' }); }}>+ 新建属性</button>
            <button className="btn-ghost" onClick={onClose}>完成</button>
          </div>
        </>
      )}

      {mode.kind !== 'list' && (
        <>
          <div className="fld">
            <label htmlFor="a-name">属性名称（1–2 个汉字）</label>
            <input ref={nameRef} id="a-name" type="text" maxLength={2} placeholder="例如：口才"
              value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="fld">
            <label htmlFor="a-en">英文缩写（2–4 个字母，雷达/卡片上显示）</label>
            <input id="a-en" type="text" maxLength={4} placeholder="例如：SPK"
              style={{ textTransform: 'uppercase' }}
              value={en} onChange={(e) => setEn(e.target.value)} />
          </div>
          <div className="fld">
            <label>颜色（深字 + 淡底自动配对，保证对比度）</label>
            <div className="swatch-row">
              {ATTR_PALETTE.map((p, i) => (
                <button key={p[0]} className={`swatch${colorIdx === i ? ' on' : ''}`}
                  style={{ background: p[0] }} onClick={() => setColorIdx(i)}
                  aria-label={`选颜色${i + 1}`} />
              ))}
            </div>
          </div>
          <div className="form-ops">
            <button className="btn-primary" onClick={submit}>{mode.kind === 'edit' ? '保存修改' : '创建属性'}</button>
            <button className="btn-ghost" onClick={() => setMode({ kind: 'list' })}>取消</button>
          </div>
          {editing && <p style={{ fontSize: 11, color: 'var(--text-3)', marginTop: 8 }}>正在编辑：{editing.name} · 等级与经验不受改名改色影响</p>}
        </>
      )}
    </Drawer>
  );
}
