// repo.rs

use std::io;
use std::iter::Rev;
use std::process::{Command, Output};
use std::vec::IntoIter;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Repo<'a> {
    pub(crate) url: &'a str,
    pub(crate) commit_subpath: &'a str,
    branch: &'a str,
    #[serde(skip)]
    inner: std::cell::RefCell<InnerCell>,
}

#[derive(Default)]
struct InnerCell {
    previous_hash: String,
    path: String,
}

impl<'a> Repo<'a> {
    fn get_hashes(&self) -> Result<Vec<String>, io::Error> {
        self.get_git_log_output(&["log", "--pretty=format:%H"])
    }

    fn get_recent_msgs(&self) -> Result<Vec<String>, io::Error> {
        self.get_git_log_output(&["log", "--pretty=format:%s"])
    }
    pub fn get_recent_messages(&mut self) -> Rev<IntoIter<(String, String)>> {
        let hashes = self.get_hashes().expect("Failed to get hashes from git log");
        let msg = self.get_recent_msgs().expect("Failed to get recent messages from git log");
        let latest_hash_from_origin = hashes[0].to_owned();
        let it = msg
            .into_iter()
            .zip(hashes)
            .take_while(|(_, h)| Some(h) != Some(&self.inner.borrow().previous_hash))
            .collect::<Vec<(String, String)>>()
            .into_iter()
            .rev();
        self.inner.borrow_mut().previous_hash = latest_hash_from_origin;
        return it;
    }
    fn get_git_log_output(&self, args: &[&str]) -> Result<Vec<String>, io::Error> {
        let path = &self.inner.borrow().path;
        let output = Command::new("git").current_dir(path).args(args)
            .arg(format!("origin/{}", &self.branch)).output()?;
        let vector: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .to_string()
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(vector)
    }
    pub fn clone(&mut self, path: &str, depth: u64) -> io::Result<Output> {
        self.inner.borrow_mut().path = path.to_owned();
        Command::new("git")
            .arg("clone")
            .arg(format!("--depth={}", depth))
            .arg("--filter=tree:0")
            .arg("--no-checkout")
            .arg("--single-branch")
            .arg("--no-tags")
            .arg("--branch")
            .arg(self.branch)
            .arg(self.url)
            .arg(path)
            .output()
    }
    pub fn is_cloned(&self) -> bool {
        &self.inner.borrow().path != ""
    }
    pub fn re_fetch(&self) -> io::Result<Output> {
        let path = &self.inner.borrow().path;
        Command::new("git")
            .current_dir(path)
            .arg("fetch")
            .arg("--filter=tree:0")
            .arg("--no-tags")
            .arg("--no-deepen")
            .arg("--update-shallow")
            .arg("--no-recurse-submodules")
            .output()
    }
}
