use std::{
    io::{self, BufRead, Read},
    process::Command,
};

// use serde::{Deserialize, Serialize};

use crate::sys::{BiComponents, Uids};



const ASSETS: &str = "./assets";
const GIT_NULL: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

#[derive(Debug)]
pub struct Assets {
    paths: BiComponents<String>,
    hash: String,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            // Empty commit
            // derived from "git hash-object -t tree /dev/null"
            // Source: https://stackoverflow.com/a/25064285
            hash: GIT_NULL.to_string(),
        }
    }
}

impl Assets {
    /// Uses git to poll file differences and register them.
    ///
    /// For my money, this is the best idea I've ever had.
    pub fn poll(&mut self, uids: &mut Uids) -> io::Result<()> {
        // Create a tree object from the working tree
        // This allows us to git diff without committing
        let working = if let Some(mut stdout) = Command::new("git")
            .arg("-C")
            .arg(ASSETS)
            .arg("write-tree")
            .spawn()?
            .stdout
            .take()
        {
            let mut working = String::new();
            stdout.read_to_string(&mut working)?;
            working
        } else {
            return io::Result::Err(io::Error::other("Buffer missing"));
        };

        // If the output does not look like a hash, return Err.
        if working.len() != GIT_NULL.len() {
            return io::Result::Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Git returned invalid data:\n{}", working),
            ));
        }
        // If there are no changes, do nothing.
        if working == self.hash {
            return Ok(());
        }

        // TODO: Add recovery from accidental prune
        let mut git_out = if let Some(stdout) = Command::new("git")
            // Run on the assets repo
            .arg("-C")
            .arg(ASSETS)
            // Difference...
            .arg("diff")
            // from the previous polled commit...
            .arg(&self.hash)
            // to the latest commit (not staged)
            .arg(&working)
            // NUL delimited
            .arg("-z")
            // 2 char status and file name
            // I think the 2nd char is unused here
            .arg("--name-status")
            .spawn()?
            .stdout
            .take()
        {
            // Seperate by NUL
            io::BufReader::new(stdout).split(b'\0')
        } else {
            return io::Result::Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Git returned invalid data:\n{}", working),
            ));
        };

        // For each line...
        // NOTE: This is not a for loop because the borrow would
        // disallow getting other entries since all entries are
        // seperated by NUL
        while let Some(status) = git_out.next() {
            let status = status?;
            let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
            match status[0] {
                b'A' => {
                    self.paths.insert(uids.add(), path);
                }
                b'D' => {
                    // TODO: use delete call
                    self.paths.remove_by_right(&path);
                }
                b'R' => {
                    let origin = path;
                    let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
                    let uid = self
                        .paths
                        .remove_by_right(&origin)
                        .map_or_else(|| uids.add(), |i| i.0);
                    self.paths.insert(uid, path);
                }
                b'C' => {
                    let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
                    self.paths.insert(uids.add(), path);
                }
                _ => {}
            }
        }

        // Prune all previous trees except the currently referenced tree.
        Command::new("git")
            .arg("-C")
            .arg(ASSETS)
            .arg("prune")
            .arg(&working);

        self.hash = working;

        Ok(())
    }
}
