To fix the issue, we need to add a guard to the `runWithCoverage()` function to prevent it from starting while a normal test run is already active. We can do this by checking the `isRunning` state before calling `setState("isRunning", true)`.

**File:** `/src/context/TestingContext.tsx`
**Lines:** `1503-1512`

```typescript
// Add a guard to prevent runWithCoverage from starting while a normal test run is already active
if (state.isRunning) {
  console.log("Cannot start coverage run while tests are already running");
  return;
}

// Rest of the function remains the same
setState("isRunning", true);
setState("output", []);
```

Alternatively, we can also modify the `CommandContext.tsx` file to dispatch a different event when the **Run Tests with Coverage** command is triggered while a normal test run is already active.

**File:** `/src/context/CommandContext.tsx`
**Lines:** `2800-2805`

```typescript
// Check if a normal test run is already active before dispatching the coverage event
if (testingContext.state.isRunning) {
  console.log("Cannot start coverage run while tests are already running");
  return;
}

// Dispatch the coverage event
dispatch({ type: "testing:run-coverage" });
```

By adding this guard, we can prevent the `runWithCoverage()` function from starting while a normal test run is already active, which will fix the issue of the shared running state being corrupted.

**Commit Message:**
```
Fix: Prevent runWithCoverage from starting while tests are already running

* Added a guard to runWithCoverage to check if a normal test run is already active
* Modified CommandContext to dispatch a different event when Run Tests with Coverage is triggered while tests are already running
```