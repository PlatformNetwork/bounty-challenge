To fix the issue, we need to add a guard to prevent `runWithCoverage()` from starting while a normal test run is already active. We can achieve this by checking the `isRunning` state before calling `runWithCoverage()`.

**Modified file:** `/src/context/TestingContext.tsx`

**Modified lines:**

* Add a check for `isRunning` before calling `runWithCoverage()`:
```typescript
// /src/context/TestingContext.tsx:1503-1512
if (state.isRunning) {
  console.log('Tests are already running. Please wait for the current run to finish.');
  return;
}
runWithCoverage();
```

**Modified file:** `/src/context/CommandContext.tsx`

**Modified lines:**

* Add a check for `isRunning` before dispatching the `testing:run-coverage` event:
```typescript
// /src/context/CommandContext.tsx:2800-2805
if (testingContext.state.isRunning) {
  console.log('Tests are already running. Please wait for the current run to finish.');
  return;
}
dispatch({ type: 'testing:run-coverage' });
```

**Alternative solution:**

Instead of blocking the `Run Tests with Coverage` command, we could queue the coverage run to start after the current test run finishes. This would require modifying the `TestingContext` to keep track of pending coverage runs and start them when the current run completes.

**Modified file:** `/src/context/TestingContext.tsx`

**Modified lines:**

* Add a `pendingCoverageRuns` array to store pending coverage runs:
```typescript
const [pendingCoverageRuns, setPendingCoverageRuns] = useState([]);
```

* Modify the `runWithCoverage()` function to add the coverage run to the `pendingCoverageRuns` array if a test run is already active:
```typescript
if (state.isRunning) {
  setPendingCoverageRuns([...pendingCoverageRuns, () => runWithCoverage()]);
  return;
}
runWithCoverage();
```

* Modify the `onTestRunFinish()` function to start the next pending coverage run:
```typescript
if (pendingCoverageRuns.length > 0) {
  const nextCoverageRun = pendingCoverageRuns.shift();
  nextCoverageRun();
}
```

This solution would allow the `Run Tests with Coverage` command to start the coverage run after the current test run finishes, without blocking the command or corrupting the shared frontend run state.