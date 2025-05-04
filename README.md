# Unititled Rust Engine



## Important Notes
<details><summary>Do not use <code>git prune</code> on your assets folder</summary>
URE uses git to track changes to your assets folder. To do this it uses <code>git write-tree</code> which creates <i>dangling trees</i> which capture the state of the repo like a commit. This allows URE to perform <code>git diff</code> arbitrarily to track files and keep UIDs in sync. After a change, URE <b>already runs <code>git prune</code></b> to remove previous trees, but URE <b>needs</b> to save the most recent tree for future reference.</details>