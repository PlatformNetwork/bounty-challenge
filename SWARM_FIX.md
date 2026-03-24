To fix the issue of Interactive Rebase including merge commits in the plan and generating a todo that Git rejects with `no -m option`, we need to modify the `git_rebase_commits` function to exclude merge commits from the plan. We can do this by adding a filter to exclude commits with more than one parent.

Here's the modified code:

```rust
// /src-tauri/src/git/rebase.rs:87-127
pub fn git_rebase_commits(onto: &str, head: &str) -> Vec<Commit> {
    let mut revwalk = Revwalk::new(git2::Repository::open(".").unwrap());
    revwalk.push(head).unwrap();
    revwalk.hide(onto).unwrap();
    let mut commits = Vec::new();

    for id in revwalk {
        let commit = git2::Commit::lookup(&git2::Repository::open(".").unwrap(), id.unwrap()).unwrap();
        if commit.parent_count() == 1 { // exclude merge commits
            commits.push(Commit {
                hash: commit.id().to_string(),
                message: commit.message().unwrap().to_string(),
            });
        }
    }

    commits
}
```

Additionally, we need to modify the `git_rebase_start` function to handle merge commits correctly. We can do this by adding a special case for merge commits and using the `git merge` command instead of `git cherry-pick`.

```rust
// /src-tauri/src/git/rebase.rs:273-297
pub fn git_rebase_start(commits: Vec<Commit>) -> Result<(), String> {
    let mut todo = String::new();

    for commit in commits {
        if commit.parent_count() > 1 { // handle merge commits
            todo.push_str(&format!("merge -C {} {}\n", commit.hash, commit.message));
        } else {
            todo.push_str(&format!("pick {} {}\n", commit.hash, commit.message));
        }
    }

    // write todo to file and start rebase
    let mut file = File::create("git-rebase-todo").unwrap();
    file.write_all(todo.as_bytes()).unwrap();
    let output = Command::new("git")
        .arg("rebase")
        .arg("--interactive")
        .arg("--onto")
        .arg("HEAD")
        .arg("--state")
        .arg("git-rebase-todo")
        .output()
        .unwrap();

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

With these changes, the Interactive Rebase feature should now correctly exclude merge commits from the plan and handle them properly when starting the rebase.