module.exports = {
  root: true,
  extends: ['eslint:recommended', 'prettier'],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  parserOptions: {
    sourceType: 'module',
    ecmaVersion: 2020,
    extraFileExtensions: ['.svelte'],
  },
  env: {
    browser: true,
    es2017: true,
    node: true,
  },
  overrides: [
    {
      files: ['*.svelte'],
      parser: 'svelte-eslint-parser',
      plugins: ['svelte3'],
      extends: [
        'eslint:recommended',
        'plugin:svelte/recommended',
      ],
      rules: {
        // 允许 Tailwind CSS 任意值语法
        'svelte/no-at-html-tags': 'off',
        'no-unused-vars': [
          'warn',
          {
            argsIgnorePattern: '^\\$',
            varsIgnorePattern: '^\\$',
          },
        ],
      },
    },
    {
      files: ['*.ts', '*.tsx', '*.js', '*.jsx'],
      extends: [
        'eslint:recommended',
        'plugin:@typescript-eslint/recommended',
      ],
    },
  ],
  ignorePatterns: [
    '*.cjs',
    'node_modules/',
    'build/',
    '.svelte-kit/',
    'dist/',
  ],
};
