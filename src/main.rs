use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use dbus::arg::messageitem::{MessageItem, MessageItemArray};
use dbus::blocking::{BlockingSender, Connection};

use crate::repo::Repo;
use crate::throttle::Throttle;

mod repo;
mod throttle;

/*
*  cargo run --release -- --update-interval=150 \
--throttle-interval=3 --max-notifications=5 --history-depth=50 \
--repos='{"url": "https://github.com/llvm/llvm-project", "commit_subpath":"/commit/", "branch": "main"}' \
--repos='{"url": "https://chromium.googlesource.com/chromium/src", "commit_subpath": "/+/", "branch": "main"}'
*/

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
        .expect("Error setting Ctrl-C handler");
    let conn = Connection::new_session().expect("Failed to connect to the D-Bus session bus");
    let mut args = parse_args();
    let mut repos = args
        .repos
        .iter()
        .map(|repo| serde_json::from_str::<Repo>(repo).expect("Failed to parse JSON: --repo parameter is invalid"))
        .collect::<Vec<Repo>>();
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let mut throttle = Throttle::new(Duration::from_secs(args.throttle_interval), args.max_notifications);
    while running.load(Ordering::SeqCst) {
        let rptmp = RepoTempPath::init(&tmp);
        for repo in repos.iter_mut() {
            if repo.is_cloned() {
                repo.re_fetch().expect("Failed to re-fetch repo");
            } else {
                let path = rptmp.new_path();
                repo.clone(&path, args.history_depth)
                    .expect("Failed to clone repo");
            }
            let msg = repo.get_recent_messages();
            let count = msg.len();
            println!("\n\n\n{} new messages for: {}\n", count, repo.url);
            for (msg, h) in msg {
                notify(&repo, &msg, &h, &conn, &mut throttle);
            }
        }
        thread::sleep(Duration::from_secs(args.update_interval));
    }
    tmp.close().expect("Failed to close temp dir");
}

fn notify(repo: &Repo, message: &str, hash: &str, conn: &Connection, throttler: &mut Throttle) {
    let commit_url = format!("{}{}{}", repo.url, repo.commit_subpath, hash);
    let message_with_link = format!("{} <a href=\"{}\">commit link</a>", message, commit_url);
    if throttler.should_allow() {
        display_notification(repo.url, &message_with_link, conn);
    }
    print!("{}\n{}\n\n", message, commit_url);
}

fn display_notification(repo: &str, message: &str, conn: &Connection) {
    // Create a method call message
    let mut msg = dbus::Message::new_method_call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
    )
        .expect("Failed to create D-Bus method call message");

    // Add the method call arguments
    msg.append_items(&[
        MessageItem::Str("Git notifier".to_owned()), // app name
        MessageItem::UInt32(0),                      // notification to update
        MessageItem::Str("".to_owned()),             // icon
        MessageItem::Str(repo.to_string()),          // summary (title)
        MessageItem::Str(message.to_string()),       // body
        MessageItem::Array(MessageItemArray::new(vec![], "as".into()).unwrap()), //actions
        MessageItem::Array(MessageItemArray::new(vec![], "a{sv}".into()).unwrap()), //hints
        MessageItem::Int32(5000),                    // timeout
    ]);
    let mut reply = conn
        .send_with_reply_and_block(msg, Duration::from_secs(1000))
        .expect("Failed to send D-Bus method call");

    // Check if the method call was successful
    if let Err(err) = reply.as_result() {
        println!("Error: {:?}", err);
    }
}

pub fn parse_args() -> Args {
    // Set default values
    let default_history_depth = 50;
    let default_update_interval = 150;
    let default_throttle_interval = 5;
    let default_max_notifications = 5;

    let matches = App::new("Parameter Parser")
        .arg(
            Arg::with_name("repos")
                .long("repos")
                .takes_value(true)
                .multiple(true)
                .required(true)
                .help("Repository information"),
        )
        .arg(
            Arg::with_name("history-depth")
                .long("history-depth")
                .takes_value(true)
                .help("Depth of commit history"),
        )
        .arg(
            Arg::with_name("update-interval")
                .long("update-interval")
                .takes_value(true)
                .help("Update interval in seconds"),
        )
        .arg(
            Arg::with_name("throttle-interval")
                .long("throttle-interval")
                .takes_value(true)
                .help("Throttle interval in seconds"),
        )
        .arg(
            Arg::with_name("max-notifications")
                .long("max-notifications")
                .takes_value(true)
                .help("Max notifications per throttle interval"),
        )
        .get_matches();

    Args {
        repos: matches.values_of("repos")
            .expect("Error: --repo arg is required")
            .map(|s| s.to_owned()).collect(),

        history_depth: matches
            .value_of("history_depth")
            .map(|s| u64::from_str(s).expect("Failed to parse history_depth"))
            .unwrap_or(default_history_depth),

        update_interval: matches
            .value_of("update_interval")
            .map(|s| u64::from_str(s).expect("Failed to parse update_interval"))
            .unwrap_or(default_update_interval),

        throttle_interval: matches
            .value_of("throttle_interval")
            .map(|s| u64::from_str(s).expect("Failed to parse throttle_interval"))
            .unwrap_or(default_throttle_interval),

        max_notifications: matches
            .value_of("max_notifications")
            .map(|s| u64::from_str(s).expect("Failed to parse max_notifications"))
            .unwrap_or(default_max_notifications),
    }
}

pub struct Args {
    pub repos: Vec<String>,
    pub history_depth: u64,
    pub update_interval: u64,
    pub throttle_interval: u64,
    pub max_notifications: u64,
}

pub struct RepoTempPath {
    tmp_dir: String,
    idx: std::cell::Cell<usize>,
}

impl RepoTempPath {
    fn new_path(&self) -> String {
        let mut path = String::new();
        path.push_str(&self.tmp_dir);
        path.push_str("/git-notifier/");
        path.push_str(&self.idx.get().to_string());
        self.idx.set(self.idx.get() + 1);
        path
    }
    fn init(tmp: &tempfile::TempDir) -> Self {
        Self {
            idx: std::cell::Cell::new(0),
            tmp_dir: tmp.path().to_str().unwrap().to_owned(),
        }
    }
}
