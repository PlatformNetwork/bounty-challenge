```rust
// Modified code in src-tauri/src/git/rebase.rs
use libgit2::{Revwalk, Sort};

// ...

let mut revwalk = Revwalk::new(repo)?;
revwalk.push(head_oid)?;
revwalk.hide(onto_oid)?;
revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?; // Add sorting

// ...
```