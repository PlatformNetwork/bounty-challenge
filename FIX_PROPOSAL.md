To fix the issue, you need to replace the "hard-drive" icon with a "memory" icon in the `SystemSpecs.tsx` file. 

Here is the exact code fix:

```tsx
// Replace this line:
<Icon name="hard-drive" />

// With this:
<Icon name="memory" />
```

This change should be made on line 281 of the `SystemSpecs.tsx` file. 

After making this change, the Memory row in the specs list should use the correct RAM-style memory icon instead of the hard-drive icon.