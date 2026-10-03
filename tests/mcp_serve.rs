//! Runs the built `scraps` binary: how a process answers a signal cannot be
//! observed from inside the test process.
#![cfg(unix)]

use std::process::Stdio;
use std::time::Duration;

use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

/// Bounds every wait, so a server that hangs fails the test instead of
/// stalling the run.
const TIMEOUT: Duration = Duration::from_secs(10);

/// SIGTERM is what `kill` and a container runtime send to stop a process.
#[tokio::test]
async fn test_http_server_exits_cleanly_on_sigterm() {
    let wiki = TempDir::new().unwrap();
    std::fs::write(wiki.path().join(".scraps.toml"), "").unwrap();

    let mut server = Command::new(env!("CARGO_BIN_EXE_scraps"))
        .arg("-C")
        .arg(wiki.path())
        .args(["mcp", "serve", "--http", "127.0.0.1:0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();

    let mut log = BufReader::new(server.stderr.as_mut().unwrap()).lines();
    timeout(TIMEOUT, async {
        let mut seen = Vec::new();
        while let Some(line) = log.next_line().await.unwrap() {
            if line.contains("listening on") {
                return;
            }
            seen.push(line);
        }
        panic!(
            "the server exited before it started listening; its log:\n{}",
            seen.join("\n")
        );
    })
    .await
    .expect("the server did not start listening in time");

    let pid = server.id().unwrap().to_string();
    let kill = Command::new("kill")
        .args(["-TERM", &pid])
        .status()
        .await
        .unwrap();
    assert!(kill.success());

    let output = timeout(TIMEOUT, server.wait_with_output())
        .await
        .expect("the server did not exit in time")
        .unwrap();
    assert!(
        output.status.success(),
        "expected a clean exit, got {}; the rest of its log:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
