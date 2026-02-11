# Theme Switcher Feature Documentation

## Overview

The Theme Switcher is a comprehensive UI component located in the status bar (bottom-left corner) that provides four main customization options:

1. **Dark Mode Toggle** - Switch between light and dark themes
2. **Language Selector** - Switch between English and Chinese
3. **Accent Color Picker** - Choose from 8 predefined accent colors

## Features

### 1. Dark Mode Toggle

- **Icon**: Sun (☀️) for light mode, Moon (🌙) for dark mode
- **Functionality**: Toggle between light and dark color schemes
- **Persistence**: Saves preference to `localStorage` as `dark-mode`
- **Instant Apply**: Changes take effect immediately without page reload

### 2. Language Selector

- **Icon**: Language/Globe icon
- **Supported Languages**:
  - 🇺🇸 English
  - 🇨🇳 中文 (Chinese)
- **Functionality**:
  - Dropdown menu with language options
  - Shows current language with checkmark
  - Displays native language names
- **Persistence**: Saves preference to `localStorage` as `xuan-brain-locale`
- **i18n Integration**: Uses Vue I18n for internationalization

### 3. Accent Color Picker

- **Icon**: Palette icon
- **Available Colors**:
  - 🔵 Blue (#3b82f6)
  - 🟣 Purple (#a855f7)
  - 🩷 Pink (#ec4899)
  - 🔴 Red (#ef4444)
  - 🟠 Orange (#f97316)
  - 🟢 Green (#22c55e)
  - 🟦 Teal (#14b8a6)
  - 🔷 Cyan (#06b6d4)
- **Functionality**:
  - Grid layout (4x2) of color swatches
  - Selected color shows checkmark overlay
  - Hover effect with scale animation
  - Border highlight on selected color
- **Persistence**: Saves preference to `localStorage` as `accent-color`
- **CSS Variables**: Updates `--accent-color` and `--accent-color-hover` CSS custom properties

## Implementation Details

### File Structure

```
src/
├── lib/
│   └── i18n/
│       ├── index.ts        # Vue I18n setup
│       ├── en.ts           # English translations
│       └── zh.ts           # Chinese translations
└── components/
    ├── ThemeSwitcher.vue   # Main theme switcher component
    └── StatusBar.vue       # Status bar with theme switcher integration
```

### Key Components

#### ThemeSwitcher.vue

Main component that handles:

- Dark mode toggle logic
- Language switching with Vue I18n
- Accent color selection and application
- Menu state management (Vuetify menus or native dialogs)

#### i18n System

- **Setup**: `createI18n()` initialized in app entry and provided via Vue app
- **Usage**: `useI18n()` composition function provides `t()` and `locale`
- **Translation Keys**: Dot notation (e.g., `t('theme.darkMode')`)
- **Fallback**: English is the fallback language if translation not found

### LocalStorage Keys

| Key                 | Value                         | Description             |
| ------------------- | ----------------------------- | ----------------------- |
| `dark-mode`         | `"true"` or `"false"`         | Dark mode preference    |
| `xuan-brain-locale` | `"en"` or `"zh"`              | Language preference     |
| `accent-color`      | Hex color (e.g., `"#3b82f6"`) | Accent color preference |

### CSS Integration

The theme system uses CSS custom properties for dynamic styling:

```css
:root {
  --accent-color: #3b82f6;
  --accent-color-hover: #2563eb;
}
```

These variables can be used throughout the application for consistent theming.

### HTML Attributes

- `data-mode="dark"` or `data-mode="light"` on `<html>` element
- `lang="en"` or `lang="zh"` on `<html>` element
- `dir="ltr"` (RTL support ready for future languages)

## Usage

### For Users

1. **Change Dark Mode**:
   - Click the sun/moon icon in the status bar
   - Theme switches instantly

2. **Change Language**:
   - Click the language icon
   - Select desired language from dropdown
   - UI text updates immediately

3. **Change Accent Color**:
   - Click the palette icon
   - Choose a color from the grid
   - Primary color updates throughout the app

### For Developers

#### Using Translations (Vue)

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t, locale } = useI18n();
</script>

<template>
  <div>
    <h1>{{ t('navigation.library') }}</h1>
    <p>Current locale: {{ locale }}</p>
  </div>
</template>
```

#### Adding New Translation Keys

1. Add to `src/lib/i18n/en.ts`:

```typescript
export default {
  mySection: {
    myKey: 'My English Text',
  },
};
```

2. Add to `src/lib/i18n/zh.ts`:

```typescript
export default {
  mySection: {
    myKey: '我的中文文本',
  },
};
```

3. Use in component:

```vue
{{ t('mySection.myKey') }}
```

#### Using Accent Color (CSS variables)

```vue
<template>
  <div class="accent-box">Styled with accent color</div>
</template>

<style scoped>
.accent-box {
  background-color: var(--accent-color);
}
.accent-box:hover {
  background-color: var(--accent-color-hover);
}
</style>
```

## Anti-Flash Initialization

To prevent theme flickering on page load, the `index.html` includes an inline script that:

1. Reads theme preferences from `localStorage`
2. Applies them to the `<html>` element immediately
3. Runs before Vue app initialization

This ensures the correct theme is visible before JavaScript loads.

## Browser Compatibility

- **Modern Browsers**: Full support (Chrome, Firefox, Safari, Edge)
- **LocalStorage**: Required for persistence
- **CSS Custom Properties**: Required for accent colors
- **MutationObserver**: Used for theme synchronization

## Future Enhancements

- [ ] Add more color themes (not just accent colors)
- [ ] Support for additional languages (Japanese, Korean, etc.)
- [ ] System theme detection (prefers-color-scheme)
- [ ] Theme import/export
- [ ] Custom color picker for accent colors
- [ ] Animation preferences (reduced motion)
- [ ] Font size scaling

## Technical Notes

### Performance

- Theme changes are instant (no page reload)
- LocalStorage operations are synchronous but minimal
- CSS custom properties enable efficient re-styling
- MutationObserver efficiently watches for `data-mode` changes

### Accessibility

- Proper ARIA labels on icon buttons
- Keyboard navigation support in menus
- Sufficient color contrast in both themes
- Screen reader friendly language names

### State Management

- Dark mode: Local component state/Pinia + DOM attribute synchronization
- Language: Vue I18n
- Accent color: Local component state + CSS custom properties
- All preferences: Persisted to `localStorage`

## Troubleshooting

### Theme Not Persisting

Check browser's `localStorage`:

```javascript
localStorage.getItem('dark-mode');
localStorage.getItem('xuan-brain-locale');
localStorage.getItem('accent-color');
```

### Translations Not Working

1. Ensure Vue I18n is initialized and provided to the app
2. Check translation keys match file structure
3. Verify locale files are imported correctly

### Accent Color Not Applying

1. Check CSS custom property is set:
   ```javascript
   getComputedStyle(document.documentElement).getPropertyValue('--accent-color');
   ```
2. Ensure components use `var(--accent-color)` in styles

## References

- [Vuetify Theming](https://vuetifyjs.com/en/features/theme/)
- [Vue I18n](https://vue-i18n.intlify.dev/)
- [CSS Custom Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
- [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)