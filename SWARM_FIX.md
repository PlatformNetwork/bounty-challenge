To fix the issue, we need to add a guard to prevent `runWithCoverage()` from starting while a normal test run is already active. We can achieve this by checking the `isRunning` state before calling `runWithCoverage()`.

**File:** `/src/context/TestingContext.tsx`

**Changes:**

1. Add a check for `isRunning` in the `runWithCoverage()` function:
```diff
- /src/context/TestingContext.tsx:1503-1512
+ /src/context/TestingContext.tsx:1503-1512
  runWithCoverage() {
+   if (state.isRunning) {
+     console.log('Cannot start coverage run while tests are already running.');
+     return;
+   }
    // ... rest of the function remains the same
  }
```
2. Alternatively, you can also add a check in the `CommandContext.tsx` file where the `testing:run-coverage` event is dispatched:
```diff
- /src/context/CommandContext.tsx:2800-2805
+ /src/context/CommandContext.tsx:2800-2805
  // ...
  case 'testing:run-coverage':
+   if (testingContext.state.isRunning) {
+     console.log('Cannot start coverage run while tests are already running.');
+     return;
+   }
    testingContext.runWithCoverage();
    break;
  // ...
```
**Explanation:**

By adding a check for `isRunning` in the `runWithCoverage()` function or in the `CommandContext.tsx` file, we ensure that the coverage run is not started while a normal test run is already active. This prevents the shared running/output state from being corrupted and ensures that the UI reflects the correct testing state.

**Tests and Example Uses:**

To test the fix, follow the steps to reproduce the issue:

1. Open a project with a test suite that takes long enough to keep the run active for a moment.
2. Start a normal test run.
3. Before it finishes, invoke **Run Tests with Coverage**.
4. Observe the test output and running indicator during and after the overlap.

With the fix, the **Run Tests with Coverage** command should be blocked or queued while the normal test run is still active, and the UI should not show an inconsistent testing state.