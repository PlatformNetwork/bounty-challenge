To fix the issue of `TestingContext` not updating after a same-window workspace switch, we need to make the following changes:

### Step 1: Update `TestingContext` to listen for `workspace:open-folder` event

In `/src/context/TestingContext.tsx`, add an event listener for `workspace:open-folder`:
```typescript
// Add this line to the useEffect hook
useEffect(() => {
  // ...
  window.addEventListener('workspace:open-folder', handleWorkspaceSwitch);
  return () => {
    window.removeEventListener('workspace:open-folder', handleWorkspaceSwitch);
  };
}, []);

// Add this function to handle the workspace switch event
const handleWorkspaceSwitch = () => {
  const newProjectPath = getProjectPath();
  discoverTests(newProjectPath);
  setState({ projectPath: newProjectPath });
};
```

### Step 2: Update `discoverTests` to detect the framework and update the state

In `/src/context/TestingContext.tsx`, update the `discoverTests` function to detect the framework and update the state:
```typescript
const discoverTests = async (projectPath: string) => {
  // ...
  const framework = await detectFramework(projectPath);
  setState({ framework, projectPath });
  // ...
};
```

### Step 3: Update `runWithCoverage` to use the updated state

In `/src/context/TestingContext.tsx`, update the `runWithCoverage` function to use the updated state:
```typescript
const runWithCoverage = async () => {
  const { projectPath, framework } = state;
  // Use the updated projectPath and framework
  // ...
};
```

### Step 4: Clear the old coverage state when the workspace switches

In `/src/context/TestingContext.tsx`, add a function to clear the old coverage state when the workspace switches:
```typescript
const clearOldCoverageState = () => {
  setState({ coverageState: null });
};
```

Call this function in the `handleWorkspaceSwitch` function:
```typescript
const handleWorkspaceSwitch = () => {
  clearOldCoverageState();
  const newProjectPath = getProjectPath();
  discoverTests(newProjectPath);
  setState({ projectPath: newProjectPath });
};
```

With these changes, `TestingContext` should now update correctly after a same-window workspace switch, and `Run Tests with Coverage` should target the newly opened workspace.

**Commit message:**
```
Fix: Update TestingContext to follow same-window workspace switches

* Add event listener for workspace:open-folder event
* Update discoverTests to detect framework and update state
* Update runWithCoverage to use updated state
* Clear old coverage state when workspace switches
```