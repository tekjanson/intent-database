use reqwest::Client;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn build_server() {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("web_server")
        .arg("--features")
        .arg("gemini-model-adapter")
        .status()
        .expect("cargo build");
    assert!(status.success());
}

fn spawn_server(port: u16) -> std::process::Child {
    build_server();
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
async fn test_server_default_is_gemini() {
    let port = 4205u16;
    let mut child = spawn_server(port);
    // wait for server to start
    sleep(Duration::from_millis(600));

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);
    let res = client.get(format!("{}/", base)).send().await.expect("get /");
    assert!(res.status().is_success());
    let body = res.text().await.expect("text");
    // server injects the default adapter into the HTML as {{DEFAULT_ADAPTER}}
    assert!(
        body.contains("{{DEFAULT_ADAPTER}}") == false,
        "server should inject default placeholder"
    );
    assert!(body.contains("gemini"), "html should reference gemini as default");

    let _ = child.kill();
}
