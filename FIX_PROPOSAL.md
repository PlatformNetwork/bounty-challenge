To fix the issue of the filename truncating due to a hardcoded `max-width` of `128px` in the Changes panel of Vibe mode, you can modify the CSS to make the filename span responsive. Here's the exact code fix:

```css
/* Remove the hardcoded max-width */
.changes-panel .filename {
  max-width: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Make the filename span responsive */
.changes-panel .filename {
  width: 100%;
  box-sizing: border-box;
}

/* Alternatively, you can use a more flexible layout */
.changes-panel {
  display: flex;
  flex-direction: row;
}

.changes-panel .filename {
  flex-grow: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
```

This code removes the hardcoded `max-width` and makes the filename span responsive by setting its width to `100%` or using a flexible layout with `flex-grow: 1`. This will allow the filename to resize based on the available panel width, preventing truncation and making it easier to identify the full changed file name.

**Commit Message:**
`Fix filename truncation in Vibe mode Changes panel by making filename span responsive`