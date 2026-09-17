import { globalIgnores } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  globalIgnores([
    '**/node_modules/**',
    '**/dist/**',
    '**/.next/**',
    '**/.next-*/**',
    // Next.js 静态导出产物（apps/web/out），是生成物，不该被 lint
    '**/out/**',
    // 打包时由 scripts/copy-sidecars.mjs 拷进 src-tauri 的前端产物，同样是生成物。
    // 不忽略的话，只要跑过一次 build:dist，`pnpm lint` 就会被压缩 js 淹没而失败。
    '**/resources/web/**',
    '**/coverage/**',
    '**/.git/**',
    '**/*.d.ts',
    '**/life-rpg-package/**',
  ]),
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    rules: {
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
    },
  },
);
