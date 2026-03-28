To fix the issue of `TestingContext` not updating after a same-window workspace switch, we need to make the following changes:

### Step 1: Update `TestingContext` to listen for `workspace:open-folder` event

In `/src/context/TestingContext.tsx`, add an event listener for `workspace:open-folder`:
```typescript
// /src/context/TestingContext.tsx:2021-2040
useEffect(() => {
  // ...
  workspace.on('workspace:open-folder', (newProjectPath) => {
    // Update testing context with new project path
    discoverTests(newProjectPath);
  });
}, []);
```
### Step 2: Update `discoverTests` to reset `state.projectPath` and `state.framework`

In `/src/context/TestingContext.tsx`, update `discoverTests` to reset `state.projectPath` and `state.framework`:
```typescript
// /src/context/TestingContext.tsx:453-455
const discoverTests = async (projectPath: string) => {
  // Reset state.projectPath and state.framework
  setState({ projectPath: projectPath, framework: null });
  // ...
};
```
### Step 3: Update `runWithCoverage` to use updated `state.projectPath` and `state.framework`

In `/src/context/TestingContext.tsx`, update `runWithCoverage` to use the updated `state.projectPath` and `state.framework`:
```typescript
// /src/context/TestingContext.tsx:1505-1537
const runWithCoverage = async () => {
  const { projectPath, framework } = state;
  // Use updated projectPath and framework
  // ...
};
```
### Step 4: Add a check to detect framework only when `state.framework` is null

In `/src/context/TestingContext.tsx`, update the framework detection to only run when `state.framework` is null:
```typescript
// /src/context/TestingContext.tsx:458-460
if (!state.framework) {
  await detectFramework(state.projectPath);
}
```
With these changes, `TestingContext` should now update correctly after a same-window workspace switch, and `Run Tests with Coverage` should target the newly opened workspace.

**Commit Message:**
```
Fix: Update TestingContext to follow same-window workspace switch

* Add event listener for workspace:open-folder
* Update discoverTests to reset state.projectPath and state.framework
* Update runWithCoverage to use updated state.projectPath and state.framework
* Add check to detect framework only when state.framework is null
```
**Files Changed:**

* `/src/context/TestingContext.tsx`