// 客户端数据访问：仅调用本地 RPC 端点，一律由服务端结算/派生（前端不重算，§4.2）。
import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
  type QueryKey,
} from '@tanstack/react-query';

export type RpcResp<T> =
  { ok: true; data: T } | { ok: false; error: { code: string; message: string } };

/** Rust 引擎端口（soloup-server）。 */
export const API_PORT = 8787;
/** MCP 服务端口（soloup-mcp）。 */
export const MCP_PORT = 8788;

/**
 * 按「当前网页所在的主机」拼出后端地址。
 *
 * 为什么不能写死 localhost：手机打开页面时，localhost 指的是**手机自己**，
 * 永远连不到你的电脑。所以统一用网页当前的 hostname 反推后端：
 * - 页面由后端托管（:8787）→ 得到同源地址；
 * - 页面来自 next dev（:3000）→ 得到同主机的 8787；
 * - Tauri 壳（tauri.localhost / file: 等伪 origin）→ 回落到 localhost。
 * 仍然可以用 NEXT_PUBLIC_SOLOUP_API 强制覆盖（优先级最高）。
 */
export function hostOf(port: number): string {
  const fallback = `http://localhost:${port}`;
  if (typeof window === 'undefined') return fallback;
  const { protocol, hostname } = window.location;
  // 只认真正的 http(s) 页面；tauri: / file: 这类伪协议一律回落
  if (protocol !== 'http:' && protocol !== 'https:') return fallback;
  if (!hostname || hostname === 'tauri.localhost') return fallback;
  return `${protocol}//${hostname}:${port}`;
}

/** Rust 引擎地址（soloup-server）。 */
export const API_BASE = process.env.NEXT_PUBLIC_SOLOUP_API ?? `${hostOf(API_PORT)}/api/rpc`;

export async function rpc<T = unknown>(op: string, args: Record<string, unknown> = {}): Promise<T> {
  const res = await fetch(API_BASE, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ op, args }),
  });
  const j = (await res.json()) as RpcResp<T>;
  if (!j.ok) throw new Error(j.error.message || j.error.code);
  return j.data;
}

export const qk = {
  meta: ['meta'] as const,
  overview: ['overview'] as const,
  tree: ['tree'] as const,
  attrs: ['attributes'] as const,
  settings: ['settings'] as const,
  audit: ['audit'] as const,
  life: ['life'] as const,
  skill: (id: string) => ['skill', id] as const,
};

/** 统一查询封装（react-query v5） */
export function useLocalQuery<T>(
  key: QueryKey,
  op: string,
  args: Record<string, unknown> = {},
  enabled = true,
) {
  return useQuery<T>({
    queryKey: key,
    queryFn: () => rpc<T>(op, args),
    enabled,
    placeholderData: keepPreviousData,
    refetchOnWindowFocus: false,
    staleTime: 8_000,
  });
}

/** 写操作封装：成功即失效（重取），返回最新一次写入结果 */
export function useLocalMutation<T = unknown>(op: string, invalidates: QueryKey[] = []) {
  const qc = useQueryClient();
  return useMutation<T, Error, Record<string, unknown>>({
    mutationFn: (args) => rpc<T>(op, args),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.overview });
      void qc.invalidateQueries({ queryKey: qk.tree });
      for (const k of invalidates) void qc.invalidateQueries({ queryKey: k });
    },
  });
}

export { useQueryClient };

// ─── MCP 服务 API ─────────────────────────────────────────────────────────────
// 注意：MCP 目前只监听回环地址，从手机访问时这一节会显示「未启动」，属预期行为。

export async function checkMcpHealth(port = MCP_PORT): Promise<boolean> {
  try {
    const res = await fetch(`${hostOf(port)}/health`, { signal: AbortSignal.timeout(3000) });
    return res.ok;
  } catch {
    return false;
  }
}

export async function getMcpToolCount(port = MCP_PORT): Promise<number> {
  try {
    const res = await fetch(`${hostOf(port)}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/list' }),
      signal: AbortSignal.timeout(3000),
    });
    const data = await res.json();
    return data.result?.tools?.length ?? 0;
  } catch {
    return 0;
  }
}
