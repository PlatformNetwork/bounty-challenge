To fix the issue, we need to replace the hard-coded spacing and border utility classes with tokenized dialog-footer styles. We can achieve this by creating a reusable class for the dialog footer and using it in the `LaunchConfigModal` component.

**Step 1: Create a reusable class for the dialog footer**

In the `src/styles/dialogFooter.css` file (create this file if it doesn't exist), add the following code:
```css
.dialog-footer {
  gap: var(--dialog-footer-gap);
  padding: var(--dialog-footer-padding);
  border-top: var(--dialog-footer-border);
  display: flex;
  justify-content: flex-end;
  align-items: center;
}
```
**Step 2: Define the tokenized dialog footer styles**

In the `src/styles/tokens.css` file (create this file if it doesn't exist), add the following code:
```css
:root {
  --dialog-footer-gap: 2rem;
  --dialog-footer-padding: 1rem 2rem;
  --dialog-footer-border: 1px solid #ccc;
}
```
**Step 3: Update the `LaunchConfigModal` component**

In the `src/components/debugger/LaunchConfigModal.tsx` file, update the lines 397-401 to use the reusable `dialog-footer` class:
```tsx
// Replace the hard-coded classes with the reusable dialog-footer class
<div className="dialog-footer">
  <Button onClick={handleCancel}>Cancel</Button>
  <Button onClick={handleStartDebugging}>Start Debugging</Button>
</div>
```
**Step 4: Remove the hard-coded classes**

Remove the hard-coded classes `gap-2 px-4 py-3 border-t` from the `LaunchConfigModal` component.

By following these steps, we have replaced the hard-coded spacing and border utility classes with tokenized dialog-footer styles, making the footer layout consistent across all dialogs.

**Commit message:**
```
Fix: Replace hard-coded footer classes with tokenized dialog-footer styles

* Create a reusable dialog-footer class
* Define tokenized dialog footer styles
* Update LaunchConfigModal to use the reusable class
```