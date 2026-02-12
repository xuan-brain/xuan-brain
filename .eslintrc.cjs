/**
 * ESLint configuration for a Vue 3 + TypeScript project.
 * - Removes React/Svelte-specific rules and overrides
 * - Enables recommended rules for Vue 3 and TypeScript
 * - Integrates with Prettier for formatting compatibility
 */
module.exports = {
  root: true,
  env: {
    browser: true,
    es2021: true,
    node: true,
  },

  // Use the Vue parser with TypeScript support
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    sourceType: 'module',
    ecmaVersion: 2021,
    extraFileExtensions: ['.vue'],
  },

  // Core extensions and plugins
  extends: [
    'eslint:recommended',
    'plugin:vue/vue3-recommended',
    'plugin:@typescript-eslint/recommended',
    'prettier',
  ],
  plugins: ['vue', '@typescript-eslint'],

  // General rules (keep minimal; rely on recommended sets)
  rules: {
    // Vue best practices
    'vue/no-mutating-props': 'warn',
    'vue/require-explicit-emits': 'off', // allow implicit emits for flexibility
    'vue/multi-word-component-names': 'off', // allow single-word component names
    'vue/no-unused-components': 'warn',
    'vue/html-self-closing': [
      'warn',
      {
        html: {
          void: 'always',
          normal: 'never',
          component: 'always',
        },
      },
    ],

    // TypeScript best practices
    '@typescript-eslint/no-unused-vars': [
      'warn',
      { argsIgnorePattern: '^_', varsIgnorePattern: '^_', ignoreRestSiblings: true },
    ],
    '@typescript-eslint/ban-ts-comment': ['warn', { 'ts-expect-error': 'allow-with-description' }],
    '@typescript-eslint/no-explicit-any': 'off', // allow any where pragmatic
    '@typescript-eslint/consistent-type-imports': 'warn',
  },

  // Overrides for file types
  overrides: [
    {
      files: ['*.vue'],
      rules: {
        // Ensure correct script setup usage and defineProps/defineEmits
        'vue/no-unused-vars': 'warn',
      },
    },
    {
      files: ['*.ts'],
      rules: {
        // TS-specific fine tuning
      },
    },
    {
      files: ['*.cjs', '*.js'],
      parserOptions: {
        sourceType: 'script',
      },
      env: {
        node: true,
      },
    },
  ],

  // Ignore generated and build outputs
  ignorePatterns: [
    'node_modules/',
    'dist/',
    'build/',
    'coverage/',
    '*.cjs', // config files may be ignored by default lint runs
    'src-tauri/target/',
    'src-tauri/**/gen/',
  ],
};
