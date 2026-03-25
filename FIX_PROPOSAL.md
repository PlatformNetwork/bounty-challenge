To fix the issue, you need to add a check for `event.repeat` in the `keydown` event listener. If `event.repeat` is `true`, you should ignore the event and not append the keystroke to the sequence.

Here is the exact code fix:

```typescript
// In WhichKeyContext.tsx
window.addEventListener('keydown', (event) => {
  // ...
  if (event.repeat) return; // Add this line to ignore repeat events
  const newPrefix = [...currentSequence.keystrokes, keystroke];
  // ...
});
```

This will prevent the repeat events from appending to the sequence and canceling the chord. Instead, the sequence will remain open for the next deliberate key press.