---
name: Tailwind Theme System with Settings
overview: Implement a Tailwind CSS v4 theme system with multiple color themes (light, dark, warm yellow, and a darker variant) and settings integration. The system will support interface.theme (light/dark/system), theme.light, and theme.dark settings, with the ability to build individual theme configurations later.
todos:
  - id: backend-handlers
    content: Create settings handlers and add to OpenAPI spec for SDK generation
    status: pending
  - id: theme-hook
    content: Create use-theme hook with system preference detection
    status: pending
  - id: tailwind-themes
    content: Configure Tailwind v4 themes in app.css with CSS variables
    status: pending
  - id: theme-provider
    content: Add theme provider to app root and apply theme classes
    status: pending
  - id: settings-ui
    content: Build settings UI for theme selection (interface.theme, theme.light, theme.dark)
    status: pending
  - id: sdk-generation
    content: Regenerate SDK after OpenAPI changes
    status: pending
---

# Tailwind Theme System with Settings

## Overview

Implement a comprehensive theme system using Tailwind CSS v4's CSS-based configuration. The system will support multiple color themes and integrate with the existing settings infrastructure.

## Architecture

### Settings Structure

Three settings keys will be added:

- `interface.theme` - Controls theme mode: `"light"`, `"dark"`, or `"system"`
- `theme.light` - Theme variant for light mode (e.g., `"light"`, `"warm-yellow"`)
- `theme.dark` - Theme variant for dark mode (e.g., `"dark"`, `"darker"`)

### Theme System Flow

```
Settings (DB) → Tauri Commands → OpenAPI Handlers → React Query → Theme Hook → CSS Variables → Tailwind Classes
```

## Implementation Steps

### 1. Backend: Settings Handlers and OpenAPI

**Files to modify:**

- `desktop/src-tauri/src/handlers/mod.rs` - Add settings module
- `desktop/src-tauri/src/handlers/settings.rs` - Create new handler file
- `desktop/src-tauri/src/api/openapi.rs` - Add settings endpoints to OpenAPI spec
- `desktop/src-tauri/src/lib.rs` - Register settings commands

**Changes:**

- Create `handlers/settings.rs` with `get_setting` and `set_setting` handlers (similar to existing transcription commands but exposed via OpenAPI)
- Add settings endpoints to OpenAPI spec for SDK generation
- Register settings commands in `lib.rs` invoke handler

### 2. Frontend: Theme Infrastructure

**Files to create:**

- `desktop/src/hooks/use-theme.ts` - Theme management hook
- `desktop/src/context/theme-context.tsx` - Theme context provider (optional, if needed for global state)

**Files to modify:**

- `desktop/src/app.tsx` - Wrap app with theme provider
- `desktop/src/app.css` - Add Tailwind v4 theme configuration with CSS variables

**Changes:**

- Create `use-theme.ts` hook that:
  - Reads `interface.theme`, `theme.light`, `theme.dark` from settings
  - Detects system preference using `window.matchMedia('(prefers-color-scheme: dark)')`
  - Applies appropriate theme class to document root
  - Provides theme state and setter functions
- Add theme provider to app root
- Configure Tailwind v4 themes using `@theme` directive in `app.css` with CSS custom properties

### 3. Tailwind v4 Theme Configuration

**File to modify:**

- `desktop/src/app.css`

**Changes:**

- Add `@theme` blocks for each theme variant (light, dark, warm-yellow, darker)
- Define color palettes using CSS custom properties
- Use data attributes or classes to switch themes (e.g., `[data-theme="light"]`, `[data-theme="dark"]`)
- Map existing `--color-neutral-*` variables to theme-aware variables

**Example structure:**

```css
@theme {
  /* Base theme variables */
}

[data-theme="light"] {
  /* Light theme overrides */
}

[data-theme="dark"] {
  /* Dark theme overrides */
}

[data-theme="warm-yellow"] {
  /* Warm yellow theme */
}

[data-theme="darker"] {
  /* Darker variant */
}
```

### 4. Settings UI

**File to modify:**

- `desktop/src/features/settings/settings.view.tsx`

**Changes:**

- Add theme selection UI with:
  - Dropdown/radio for `interface.theme` (light/dark/system)
  - Dropdown/radio for `theme.light` (light/warm-yellow)
  - Dropdown/radio for `theme.dark` (dark/darker)
- Use React Query mutations to update settings
- Show preview of selected theme

### 5. System Theme Detection

**Implementation:**

- In `use-theme.ts`, listen to `prefers-color-scheme` media query changes
- When `interface.theme === "system"`, automatically switch based on system preference
- Update theme when system preference changes

### 6. SDK Generation

**File to modify:**

- `desktop/orval.config.ts` (if needed)

**Changes:**

- After adding settings endpoints to OpenAPI, run `npm run generate:sdk` to regenerate TypeScript SDK
- Use generated SDK types in frontend code

## Technical Details

### Theme Switching Mechanism

- Apply theme via `data-theme` attribute on `<html>` or `#root` element
- Tailwind v4 will use CSS variables defined in theme blocks
- Theme changes trigger immediate visual update

### Default Values

- `interface.theme`: `"system"` (default)
- `theme.light`: `"light"` (default)
- `theme.dark`: `"dark"` (default)

### Color Variable Mapping

Existing color variables like `--color-neutral-600` will be mapped to theme-aware variables:

- `--color-neutral-600` → `var(--theme-neutral-600)` (theme-specific)

## Future Enhancements

- Individual theme color customization UI
- Theme preview/editor
- Export/import theme configurations
- Additional theme variants

## Notes

- Tailwind v4 uses CSS-based configuration, so no `tailwind.config.js` needed
- Settings are already encrypted for sensitive keys, but theme settings won't be encrypted
- The existing `get_setting`/`set_setting` commands in `commands/transcription.rs` can be reused or moved to a dedicated settings handler