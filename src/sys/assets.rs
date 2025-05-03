use std::{fs::read_dir, io, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use super::{BiComponents, UIDs};

const ASSETS: &str = "./assets";

#[derive(Debug, Serialize, Deserialize)]
pub struct Assets {
    paths: BiComponents<String>,
}

impl Assets {
    pub fn poll(&mut self, uids: &mut UIDs) -> io::Result<()> {
        self.poll_dir(PathBuf::from_str(ASSETS).unwrap(), uids)
    }
    fn poll_dir(&mut self, dir: PathBuf, uids: &mut UIDs) -> io::Result<()> {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.poll_dir(path, uids)?
            } else {
                let path = path.to_str().unwrap();
                if self.paths.contains_right(path) {
                    continue;
                }
                self.paths.insert(uids.add(), path.to_string());
            }
        }
        Ok(())
    }
}
