To fix the issue, we can use the `color-mix` function in CSS, which allows us to mix two colors together. This approach is more robust and works with any color syntax.

Replace the following lines in `src/components/SystemSpecs.tsx`:

```tsx
background: var(--cortex-success)20;
```

and

```tsx
background: var(--cortex-error)20;
```

with:

```tsx
background: color-mix(in srgb, var(--cortex-success) 12%, transparent);
```

and

```tsx
background: color-mix(in srgb, var(--cortex-error) 12%, transparent);
```

This will create a tinted surface that works with any color syntax the theme uses.

Alternatively, you can define a dedicated token for the background color, for example:

```css
--cortex-success-bg: color-mix(in srgb, var(--cortex-success) 12%, transparent);
--cortex-error-bg: color-mix(in srgb, var(--cortex-error) 12%, transparent);
```

Then, use these tokens in your component:

```tsx
background: var(--cortex-success-bg);
```

and

```tsx
background: var(--cortex-error-bg);
```

This approach is more maintainable and allows you to easily change the background color in the future.