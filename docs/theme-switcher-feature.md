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
- **i18n Integration**: Uses React Context for internationalization

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
│       ├── index.tsx       # i18n provider and hooks
│       ├── en.ts           # English translations
│       └── zh.ts           # Chinese translations
└── components/
    ├── ThemeSwitcher.tsx   # Main theme switcher component
    └── StatusBar.tsx       # Status bar with theme switcher integration
```

### Key Components

#### ThemeSwitcher.tsx

Main component that handles:

- Dark mode toggle logic
- Language switching with i18n context
- Accent color selection and application
- Menu state management (Material-UI Menu components)

#### i18n System

- **Provider**: `I18nProvider` wraps the entire app in `App.tsx`
- **Hook**: `useI18n()` provides translation function `t()` and locale state
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

#### Using Translations

```tsx
import { useI18n } from '../lib/i18n';

function MyComponent() {
  const { t, locale } = useI18n();

  return (
    <div>
      <h1>{t('navigation.library')}</h1>
      <p>Current locale: {locale}</p>
    </div>
  );
}
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

```tsx
{
  t('mySection.myKey');
}
```

#### Using Accent Color

The accent color is available as a CSS custom property:

```tsx
<Box
  sx={{
    backgroundColor: 'var(--accent-color)',
    '&:hover': {
      backgroundColor: 'var(--accent-color-hover)',
    },
  }}
>
  Styled with accent color
</Box>
```

## Anti-Flash Initialization

To prevent theme flickering on page load, the `index.html` includes an inline script that:

1. Reads theme preferences from `localStorage`
2. Applies them to the `<html>` element immediately
3. Runs before React hydration

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

- Dark mode: Local React state + DOM attribute synchronization
- Language: React Context (`I18nProvider`)
- Accent color: Local React state + CSS custom properties
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

1. Ensure `I18nProvider` wraps your app
2. Check translation keys match file structure
3. Verify locale files are imported correctly

### Accent Color Not Applying

1. Check CSS custom property is set:
   ```javascript
   getComputedStyle(document.documentElement).getPropertyValue('--accent-color');
   ```
2. Ensure components use `var(--accent-color)` in styles

## References

- [Material-UI Theming](https://mui.com/material-ui/customization/theming/)
- [React Context API](https://react.dev/reference/react/useContext)
- [CSS Custom Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
- [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
