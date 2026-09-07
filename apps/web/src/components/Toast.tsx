'use client';
/* ================= Toast：全局轻提示 ================= */
import React, { createContext, useCallback, useContext, useRef, useState } from 'react';

interface ToastCtx {
  toast: (msg: string) => void;
}
const Ctx = createContext<ToastCtx>({ toast: () => {} });

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [msg, setMsg] = useState('');
  const [show, setShow] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const toast = useCallback((m: string) => {
    if (timer.current) clearTimeout(timer.current);
    if (hideTimer.current) clearTimeout(hideTimer.current);
    setMsg(m);
    setShow(true);
    timer.current = setTimeout(() => setShow(false), 2600);
    hideTimer.current = setTimeout(() => setMsg(''), 3000);
  }, []);

  return (
    <Ctx.Provider value={{ toast }}>
      {children}
      <div className={`toast${show ? ' show' : ''}`} role="status">{msg}</div>
    </Ctx.Provider>
  );
}

export function useToast() {
  return useContext(Ctx).toast;
}
