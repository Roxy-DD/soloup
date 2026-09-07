// 客户端数据访问：仅调用本地 RPC 端点，一律由服务端结算/派生（前端不重算，§4.2）。
import { useQuery, useMutation, useQueryClient, keepPreviousData, type QueryKey } from '@tanstack/react-query';

export type RpcResp<T> = { ok: true; data: T } | { ok: false; error: { code: string; message: string } };

/** Rust 引擎地址（soloup-server）。默认本机 8787；可用 NEXT_PUBLIC_SOLOUP_API 覆盖。 */
export const API_BASE = process.env.NEXT_PUBLIC_SOLOUP_API ?? 'http://localhost:8787/api/rpc';

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

const MCP_PORT = 8788;

export async function checkMcpHealth(port = MCP_PORT): Promise<boolean> {
  try {
    const res = await fetch(`http://localhost:${port}/health`, { signal: AbortSignal.timeout(3000) });
    return res.ok;
  } catch {
    return false;
  }
}

export async function getMcpToolCount(port = MCP_PORT): Promise<number> {
  try {
    const res = await fetch(`http://localhost:${port}/mcp`, {
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
