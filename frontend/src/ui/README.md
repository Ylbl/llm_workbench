# Workbench UI Layer

This folder is the only place where headless UI primitives should be wrapped.

- `tokens.css` owns color, spacing, radius, and typography variables.
- `components.css` owns default visual states for `Wb*` components.
- `Wb*.vue` components are the public UI API for feature views.
- Reka UI imports belong here, not in business pages.

When adding a new interaction primitive, create a `Wb*` wrapper first and style
it with the existing tokens.
