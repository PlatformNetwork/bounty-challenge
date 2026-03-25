# Fix: Repo validator/import scan treats package-lock.json as a directory

## Related Issue

Fixes #37911

## Problem

The repo validator/import scan logic traverses every path in the project root without checking whether it is a file or directory. When it encounters `package-lock.json` (a file), it calls `readdirSync` / `scandir` / directory walk on it, producing:

```
ENOTDIR: not a directory, scandir '<...>/package-lock.json'
```

This causes the traversal to fail or produce incorrect validation results.

## Root Cause

The directory traversal function (e.g., `walkDir`, `scanRepo`, or similar recursive scan) does not call `isDirectory()` / `stat().isDir()` before recursing into child entries. It iterates over all entries from `readdirSync` and blindly recurses into each one, including regular files like `package-lock.json`.

## Fix

Add an `isDirectory()` check before recursing. Below are examples in the two most likely languages used by the scanner:

### Node.js / TypeScript

```typescript
import * as fs from 'fs';
import * as path from 'path';

function walkDir(dirPath: string, callback: (filePath: string) => void): void {
  const entries = fs.readdirSync(dirPath, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      // Only recurse into directories — skip files like package-lock.json
      walkDir(fullPath, callback);
    } else if (entry.isFile()) {
      callback(fullPath);
    }
  }
}
```

### Rust

```rust
use std::fs;
use std::path::Path;

fn walk_dir(dir: &Path, callback: &dyn Fn(&Path)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, callback)?;
            } else {
                callback(&path);
            }
        }
    }
    Ok(())
}
```

## Key Change

The critical fix is adding the directory/file check before recursion:

- **Before:** `readdirSync(dir)` → for each entry → `walkDir(entry)` (crashes on files)
- **After:** `readdirSync(dir)` → for each entry → check `isDirectory()` → only then `walkDir(entry)`; if `isFile()`, call callback

## Testing

1. Create a test project with a root `package-lock.json` file
2. Run the validator/import scan on it
3. Verify no `ENOTDIR` errors occur
4. Verify `package-lock.json` contents are correctly scanned (as a file, not traversed as directory)
5. Verify normal directories are still recursively traversed

```bash
# Create test fixture
mkdir /tmp/test-project
echo '{}' > /tmp/test-project/package-lock.json
mkdir /tmp/test-project/src
echo 'fn main() {}' > /tmp/test-project/src/main.rs

# Run validator — should complete without ENOTDIR error
cargo test -- test_package_lock_not_traversed_as_dir
```

## Impact

- **Severity:** Medium — causes scan failures on any project with a root `package-lock.json` (nearly all Node.js projects)
- **Scope:** Repo validator/import scan module
- **Breaking:** No — purely a bug fix, no API or behavior change for valid inputs
