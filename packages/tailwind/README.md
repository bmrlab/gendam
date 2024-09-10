# Tailwind CSS Custom Color Variables Documentation

This document provides an overview of the custom color variables defined in the Tailwind CSS configuration for the `gendam` project. It also explains how to use these variables in your styles.

## Color Variables

The color variables are defined in the `globals.css` file and are categorized based on different UI regions and their semantic purposes. These variables are used to maintain a consistent color scheme across the application.

### Global Semantic Colors

- **Accent Color**
  - `--color-accent`: The primary highlight color (blue in this case).

- **Text Color**
  - `--color-ink`: The default text color.

### Region-Specific Colors

#### Main App (`app`)

- `--color-app`: Background color of the main app.
- `--color-app-box`: Color for dialog boxes, menus, etc.
- `--color-app-hover`: Color for hover and selected states.
- `--color-app-line`: Color for borders and dividers.
- `--color-app-overlay`: Color for highlighted areas (e.g., input fields, alternating rows).

#### Toolbar (`toolbar`)

- `--color-toolbar`: Background color of the toolbar.
- `--color-toolbar-hover`: Color for hover states in the toolbar.
- `--color-toolbar-line`: Color for borders and dividers in the toolbar.

#### Sidebar (`sidebar`)

- `--color-sidebar`: Background color of the sidebar.
- `--color-sidebar-hover`: Color for hover states in the sidebar.
- `--color-sidebar-line`: Color for borders and dividers in the sidebar.

### Dark Mode Colors

The dark mode colors are defined within the `html.dark` selector. These variables override the default light mode colors when the dark mode is active.

## Using Color Variables

The color variables are integrated into the Tailwind CSS configuration (`tailwind.config.ts`) and can be used directly in your Tailwind classes.

### Example Usage

To use these custom color variables in your Tailwind CSS classes, you can refer to them using the defined keys. Here are some examples:

#### Background Colors

```html
<div class="bg-app">
  <!-- Main app background -->
</div>

<div class="bg-toolbar">
  <!-- Toolbar background -->
</div>

<div class="bg-sidebar">
  <!-- Sidebar background -->
</div>
```

#### Text Colors

```html
<p class="text-ink">
  <!-- Default text color -->
</p>
```

#### Hover States

```html
<button class="hover:bg-app-hover">
  <!-- Button with hover state -->
</button>

<div class="hover:bg-toolbar-hover">
  <!-- Toolbar item with hover state -->
</div>

<div class="hover:bg-sidebar-hover">
  <!-- Sidebar item with hover state -->
</div>
```

#### Borders and Dividers

```html
<hr class="border-app-line">
  <!-- Divider in the main app -->
</hr>

<hr class="border-toolbar-line">
  <!-- Divider in the toolbar -->
</hr>

<hr class="border-sidebar-line">
  <!-- Divider in the sidebar -->
</hr>
```

### Utility Classes

In addition to the color variables, some utility classes are defined in the `globals.css` file:

- `.text-balance`: Applies balanced text wrapping.
- `.no-scrollbar`: Hides the scrollbar for various browsers.

## Conclusion

By using these custom color variables, you can ensure a consistent and maintainable color scheme across your application. The variables are designed to be flexible and can be easily adjusted to fit different themes, including light and dark modes.
