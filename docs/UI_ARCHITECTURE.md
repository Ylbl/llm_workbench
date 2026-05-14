# UI Architecture

This project uses a custom IDE-style workbench design system. Do not treat a
third-party component library as the product visual layer.

## Direction

- The workbench shell is custom: activity bars, tool windows, editor tabs,
  editor surface, right-side panels, and status bar are first-party UI.
- Reka UI is the headless interaction layer for menus, dialogs, popovers,
  tooltips, tabs, and other complex keyboard/focus behavior.
- Visual styling is owned by `src/ui/tokens.css` and `src/ui/components.css`.
- Business views should consume `Wb*` components instead of importing Reka UI or
  styling raw controls directly.

## Layering

```text
business views
  -> src/ui/Wb*.vue
      -> reka-ui primitives when interaction behavior is non-trivial
      -> src/ui/tokens.css + src/ui/components.css for visual style
  -> workbench shell layout in WorkspaceShell.vue
```

## Rules For Future UI Work

- Do not import `reka-ui` directly in feature views. Wrap it in `src/ui`.
- Prefer `WbButton`, `WbIconButton`, and future `WbSelect`, `WbDialog`,
  `WbPopover`, `WbTree`, and `WbTabs` components over raw controls.
- Do not introduce a styled component library such as Element Plus, Naive UI, or
  PrimeVue into business screens unless the visual layer is fully wrapped by
  `Wb*`.
- New colors should be added as tokens first. Avoid page-local hex colors.
- Keep IDE chrome dense: compact rows, modest radii, thin borders, no marketing
  cards, and no large decorative gradients.
- When a control needs keyboard navigation, focus trapping, positioning, or ARIA
  behavior, use a Reka-backed `Wb*` wrapper instead of hand-rolling it.

## Current Foundation

- `src/ui/tokens.css`: design tokens.
- `src/ui/components.css`: base Wb component styles.
- `src/ui/WbButton.vue`: standard command button.
- `src/ui/WbIconButton.vue`: activity/tool/icon button.
- `src/ui/WbDropdownMenu.vue`: Reka-backed dropdown shell.
- `src/ui/WbMenuItem.vue`: Reka-backed menu item.

The current WorkspaceShell uses these wrappers as a trial integration while
keeping existing product behavior unchanged.
