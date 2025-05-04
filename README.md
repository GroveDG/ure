# Unititled Rust Engine
![URE logo](./logo.svg)

## Important Notes
### Do not use `git prune` on your assets folder
> URE uses git to track changes to your assets folder. To do this it uses `git write-tree` which creates a dangling tree which captures the state of the repo like a commit, but add it to the history. This allows URE to perform `git diff` arbitrarily to track files and keep UIDs in sync. After a change, URE **already runs `git prune`** to remove previous trees, but URE **needs** to save the most recent tree for future reference.