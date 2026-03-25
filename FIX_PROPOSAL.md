To fix the issue, we need to replace the Tailwind palette classes with the Cortex text tokens in the `SystemSpecs.tsx` file. 

Here's the exact code fix:

Replace the following lines:
```tsx
className="text-neutral-400"
className="text-neutral-500"
className="text-red-400"
```
With:
```tsx
className="text-secondary" // for secondary text
className="text-inactive" // for inactive text
className="text-error" // for error text
```
Or, if you want to use the Cortex CSS variable theme:
```tsx
style={{ color: 'var(--cortex-text-secondary)' }} // for secondary text
style={{ color: 'var(--cortex-text-inactive)' }} // for inactive text
style={{ color: 'var(--cortex-error)' }} // for error text
```
Make sure to update all occurrences of `text-neutral-400`, `text-neutral-500`, and `text-red-400` in the `SystemSpecs.tsx` file.

Here's an example of how the updated code might look:
```tsx
// Before
<div className="text-neutral-400">Secondary text</div>
<div className="text-neutral-500">Inactive text</div>
<div className="text-red-400">Error text</div>

// After
<div className="text-secondary">Secondary text</div>
<div className="text-inactive">Inactive text</div>
<div className="text-error">Error text</div>
```
Or:
```tsx
// Before
<div className="text-neutral-400">Secondary text</div>
<div className="text-neutral-500">Inactive text</div>
<div className="text-red-400">Error text</div>

// After
<div style={{ color: 'var(--cortex-text-secondary)' }}>Secondary text</div>
<div style={{ color: 'var(--cortex-text-inactive)' }}>Inactive text</div>
<div style={{ color: 'var(--cortex-error)' }}>Error text</div>
```