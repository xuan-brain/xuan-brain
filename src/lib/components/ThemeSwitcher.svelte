<script lang="ts">
  // Available Skeleton color themes
  const colorThemes = [
    { name: 'Cerberus', value: 'cerberus', emoji: '🤖' },
    { name: 'Catppuccin', value: 'catppuccin', emoji: '🐈' },
    { name: 'Concord', value: 'concord', emoji: '🔴' },
    { name: 'Crimson', value: 'crimson', emoji: '🦊' },
    { name: 'Fennec', value: 'fennec', emoji: '👔' },
    { name: 'HamlinDigo', value: 'hamlindigo', emoji: '💀' },
    { name: 'Legacy', value: 'legacy', emoji: '🍃' },
    { name: 'Mint', value: 'mint', emoji: '🌸' },
    { name: 'Modern', value: 'modern', emoji: '🐙' },
    { name: 'Mona', value: 'mona', emoji: '🥙' },
    { name: 'Nosh', value: 'nosh', emoji: '🎑' },
    { name: 'Nouveau', value: 'nouveau', emoji: '🌲' },
    { name: 'Pine', value: 'pine', emoji: '📒' },
    { name: 'Reign', value: 'reign', emoji: '🚀' },
    { name: 'Rocket', value: 'rocket', emoji: '🌷' },
    { name: 'Rose', value: 'rose', emoji: '🏜️' },
    { name: 'Sahara', value: 'sahara', emoji: '🏜️' },
    { name: 'Seafoam', value: 'seafoam', emoji: '🌑' },
    { name: 'Terminus', value: 'terminus', emoji: '📺' },
    { name: 'Vintage', value: 'vintage', emoji: '👾' },
    { name: 'Vox', value: 'vox', emoji: '🌨️' },
    { name: 'Wintry', value: 'wintry', emoji: '❄️' }
  ];

  // Theme state using Svelte 5 runes
  let selectedColorTheme = $state('cerberus');
  let isDarkMode = $state(false);
  let isOpen = $state(false);

  // Apply theme to document
  function applyColorTheme(theme: string) {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', theme);
    }
    if (typeof window !== 'undefined') {
      localStorage.setItem('color-theme', theme);
    }
  }

  function applyDarkMode(dark: boolean) {
    if (typeof document !== 'undefined') {
      const mode = dark ? 'dark' : 'light';
      document.documentElement.setAttribute('data-mode', mode);
    }
    if (typeof window !== 'undefined') {
      localStorage.setItem('dark-mode', String(dark));
    }
  }

  // Initialize themes from localStorage on mount
  function initializeThemes() {
    if (typeof window !== 'undefined') {
      const savedColorTheme = localStorage.getItem('color-theme') || 'cerberus';
      const savedDarkMode = localStorage.getItem('dark-mode') === 'true';

      selectedColorTheme = savedColorTheme;
      isDarkMode = savedDarkMode;

      applyColorTheme(savedColorTheme);
      applyDarkMode(savedDarkMode);
    }
  }

  // Initialize immediately on client side
  if (typeof window !== 'undefined') {
    initializeThemes();
  }

  // Apply color theme whenever it changes
  $effect(() => {
    applyColorTheme(selectedColorTheme);
  });

  // Apply dark mode whenever it changes
  $effect(() => {
    applyDarkMode(isDarkMode);
  });

  // Toggle dropdown
  function toggleDropdown() {
    isOpen = !isOpen;
  }

  // Select a theme
  function selectTheme(theme: string) {
    selectedColorTheme = theme;
    isOpen = false;
  }

  // Toggle dark mode
  function toggleDarkMode() {
    isDarkMode = !isDarkMode;
  }

  // Close dropdown when clicking outside
  function handleOutsideClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.theme-switcher')) {
      isOpen = false;
    }
  }
</script>

<!-- Prevent flash by setting themes immediately before hydration -->
<svelte:head>
  <script>
    (function () {
      const colorTheme = localStorage.getItem('color-theme') || 'cerberus';
      const darkMode = localStorage.getItem('dark-mode') === 'true';
      document.documentElement.setAttribute('data-theme', colorTheme);
      document.documentElement.setAttribute('data-mode', darkMode ? 'dark' : 'light');
    })();
  </script>
</svelte:head>

<svelte:window on:click={handleOutsideClick} />

<div class="theme-switcher">
  <!-- Dark Mode Toggle -->
  <div class="flex items-center gap-3 mb-3">
    <!-- Dark Mode Switch -->
    <button
      class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
      class:bg-gray-200={!isDarkMode}
      class:bg-blue-600={isDarkMode}
      class:dark:bg-gray-700={!isDarkMode}
      class:dark:bg-blue-600={isDarkMode}
      onclick={toggleDarkMode}
      aria-label="Toggle dark mode"
      aria-pressed={isDarkMode}
      type="button"
    >
      <span
        class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
        class:translate-x-0={!isDarkMode}
        class:translate-x-5={isDarkMode}
      ></span>
    </button>

    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
      Dark Mode
    </span>
  </div>

  <!-- Color Theme Selector -->
  <div class="relative">
    <button
      class="w-full flex items-center justify-between gap-2 px-3 py-2 text-left text-sm font-medium rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
      onclick={toggleDropdown}
      aria-expanded={isOpen}
      aria-haspopup="listbox"
    >
      <span class="flex items-center gap-2">
        <span class="text-lg">
          {colorThemes.find(t => t.value === selectedColorTheme)?.emoji || '🎨'}
        </span>
        <span>{colorThemes.find(t => t.value === selectedColorTheme)?.name || 'Theme'}</span>
      </span>
      <svg
        class={`h-5 w-5 transition-transform ${isOpen ? 'rotate-180' : ''}`}
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.02l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
          clip-rule="evenodd"
        />
      </svg>
    </button>

    <!-- Dropdown Menu -->
    {#if isOpen}
      <div
        class="absolute bottom-full left-0 mb-2 w-full max-h-80 overflow-y-auto rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50"
        role="listbox"
      >
        <div class="p-1 space-y-1">
          {#each colorThemes as theme}
            <button
              class="w-full flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors {selectedColorTheme === theme.value
                ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
              onclick={() => selectTheme(theme.value)}
              role="option"
              aria-selected={selectedColorTheme === theme.value}
            >
              <span class="text-lg">{theme.emoji}</span>
              <span>{theme.name}</span>
              {#if selectedColorTheme === theme.value}
                <svg
                  class="ml-auto h-4 w-4"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-1.5a.75.75 0 01.014-1.438l8.5-8.5a.75.75 0 011.06-.009z"
                    clip-rule="evenodd"
                  />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Ensure button has no default appearance */
  button {
    appearance: none;
    border: none;
    background: none;
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }

  /* Add subtle hover effect */
  .theme-switcher button:hover {
    filter: brightness(1.1);
  }

  .theme-switcher button:active {
    transform: scale(0.98);
  }

  /* Scrollbar styling for dropdown */
  .theme-switcher div[role="listbox"]::-webkit-scrollbar {
    width: 6px;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-track {
    background: transparent;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Dark mode scrollbar */
  @media (prefers-color-scheme: dark) {
    .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb {
      background: #4b5563;
    }

    .theme-switcher div[role="listbox"]::-webkit-scrollbar-thumb:hover {
      background: #6b7280;
    }
  }
</style>
