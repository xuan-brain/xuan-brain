/**
 * ESLint configuration for a Vue 3 + TypeScript project.
 * Uses the new flat config format for ESLint 9+.
 */
import js from '@eslint/js';
import tsParser from '@typescript-eslint/parser';
import tsPlugin from '@typescript-eslint/eslint-plugin';
import vueParser from 'vue-eslint-parser';
import vuePlugin from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default [
  // Base JavaScript recommended rules
  js.configs.recommended,

  // Vue recommended rules
  ...vuePlugin.configs['flat/recommended'],

  // Prettier compatibility (turns off conflicting rules)
  prettier,

  // Global settings for Vue files
  {
    files: ['**/*.vue'],
    languageOptions: {
      ecmaVersion: 2021,
      sourceType: 'module',
      parser: vueParser,
      parserOptions: {
        parser: tsParser,
        extraFileExtensions: ['.vue'],
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2021,
      },
    },
    plugins: {
      '@typescript-eslint': tsPlugin,
    },
    rules: {
      // TypeScript recommended rules for Vue files
      ...tsPlugin.configs.recommended.rules,
      // Vue best practices
      'vue/no-mutating-props': 'warn',
      'vue/require-explicit-emits': 'off',
      'vue/multi-word-component-names': 'off',
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
      // Disable attribute order warning
      'vue/attributes-order': 'off',
      // Disable v-slot-style warning
      'vue/v-slot-style': 'off',
      // Disable no-template-shadow warning
      'vue/no-template-shadow': 'off',
      // Disable no-v-html warning (used intentionally)
      'vue/no-v-html': 'off',
      // TypeScript
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },
  },

  // TypeScript files
  {
    files: ['**/*.ts', '**/*.tsx'],
    languageOptions: {
      ecmaVersion: 2021,
      sourceType: 'module',
      parser: tsParser,
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2021,
      },
    },
    plugins: {
      '@typescript-eslint': tsPlugin,
    },
    rules: {
      // TypeScript recommended rules
      ...tsPlugin.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
      '@typescript-eslint/ban-ts-comment': ['warn', { 'ts-expect-error': 'allow-with-description' }],
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/consistent-type-imports': 'warn',
      '@typescript-eslint/no-empty-object-type': 'off',
      // Disable preserve-caught-error rule
      'preserve-caught-error': 'off',
    },
  },

  // JavaScript files in scripts folder
  {
    files: ['scripts/**/*.js'],
    languageOptions: {
      ecmaVersion: 2021,
      sourceType: 'module',
      globals: {
        ...globals.node,
        ...globals.es2021,
      },
    },
  },

  // JavaScript/CommonJS files
  {
    files: ['**/*.cjs'],
    languageOptions: {
      ecmaVersion: 2021,
      sourceType: 'script',
      globals: {
        ...globals.node,
        ...globals.browser,
        ...globals.es2021,
      },
    },
  },

  // Config files (JS with ESM syntax)
  {
    files: ['**/eslint.config.js'],
    languageOptions: {
      ecmaVersion: 2021,
      sourceType: 'module',
      globals: {
        ...globals.node,
        ...globals.browser,
        ...globals.es2021,
      },
    },
  },

  // Ignore patterns
  {
    ignores: [
      'node_modules/**',
      'dist/**',
      'build/**',
      'coverage/**',
      'src-tauri/target/**',
      'src-tauri/**/gen/**',
    ],
  },
];
