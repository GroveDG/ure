use std::{
    io::{self, BufRead},
    process::Command,
};

use serde::{Deserialize, Serialize};

use super::{BiComponents, UIDs};

const ASSETS: &str = "./assets";

#[derive(Debug, Serialize, Deserialize)]
pub struct Assets {
    paths: BiComponents<String>,
    commit: String,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            // derived from "git hash-object -t tree /dev/null"
            // Source: https://stackoverflow.com/a/25064285
            commit: "4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string(),
        }
    }
}

impl Assets {
    /// Uses git to poll file differences and register them.
    ///
    /// For my money, this is the best idea I've ever had.
    pub fn poll(&mut self, uids: &mut UIDs) -> io::Result<()> {
        let git = Command::new("git")
            // Use assets repo
            .arg("-C")
            .arg(ASSETS)
            // Difference...
            .arg("diff")
            // from the previous polled commit...
            .arg(&self.commit)
            // to the latest commit (not staged)
            .arg("HEAD")
            // NUL delimited
            .arg("-z")
            // 2 char status and file name
            .arg("--name-status")
            .spawn()?;

        let mut git_out = io::BufReader::new(git.stdout.unwrap()).split(b'\0');

        while let Some(status) = git_out.next() {
            let status = status?;
            let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
            match status[0] {
                b'A' => {
                    self.paths.insert(uids.add(), path);
                }
                b'D' => {
                    // TODO: replace with delete entity
                    self.paths.remove_by_right(&path);
                }
                b'R' => {
                    let origin = path;
                    let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
                    let uid = self.paths.remove_by_right(&origin).map_or_else(|| uids.add(), |i| i.0);
                    self.paths.insert(uid, path);
                }
                b'C' => {
                    let path = String::from_utf8(git_out.next().unwrap()?).unwrap();
                    self.paths.insert(uids.add(), path);
                }
                _ => { }
            }
        }

        Ok(())
    }
}
