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
