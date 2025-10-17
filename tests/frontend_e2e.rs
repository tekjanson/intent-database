use reqwest::Client;
use serde_json::json;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

fn spawn_server(port: u16) -> Child {
    // Ensure the binary is built first to avoid racing with `cargo run`.
    let status = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("web_server")
        .status()
        .expect("cargo build");
    assert!(status.success());

    let exe = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join(if cfg!(windows) { "web_server.exe" } else { "web_server" });

    let mut cmd = Command::new(exe);
    cmd.env("WEB_SERVER_PORT", port.to_string());
    cmd.spawn().expect("start server")
}

#[tokio::test]
async fn web_ui_e2e() {
    let port = 4001u16;
    let mut child = spawn_server(port);

    // wait for server to start
    sleep(Duration::from_millis(500));

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Call chat endpoint directly (real-only E2E: do not mock or set keys here).
    let r2 = client
        .post(format!("{}/chat", base))
        .json(&json!({"context": "hello"}))
        .send()
        .await
        .expect("post chat");
    assert!(r2.status().is_success());
    let body: serde_json::Value = r2.json().await.expect("json");
    assert!(body.get("response").is_some());

    // kill child
    let _ = child.kill();
}
