use futures_util::StreamExt;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::net::TcpStream;

#[tokio::test]
#[ignore]
async fn streams_datastar_event() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root");

    let status = Command::new("buck2")
        .args([
            "build",
            "--target-platform=wasm32-unknown-unknown",
            "//crates/infra/datastar-edge-worker:datastar_edge_worker",
        ])
        .current_dir(repo_root)
        .status()
        .expect("failed to build worker with buck2");
    assert!(status.success(), "buck2 build failed");

    let output = Command::new("buck2")
        .args([
            "targets",
            "--show-output",
            "--target-platform=wasm32-unknown-unknown",
            "//crates/infra/datastar-edge-worker:datastar_edge_worker",
        ])
        .current_dir(repo_root)
        .output()
        .expect("failed to locate wasm artifact");
    assert!(output.status.success(), "buck2 targets failed");
    let line = String::from_utf8(output.stdout).expect("utf8");
    let rel = line.split_whitespace().nth(1).expect("no path");
    let wasm_path = repo_root.join(rel).to_string_lossy().to_string();

    let mut child = Command::new("wrangler")
        .args(["dev", &wasm_path, "--local", "--port", "8787"])
        .current_dir(manifest_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn wrangler dev");

    let mut ready = false;
    for _ in 0..300 {
        if TcpStream::connect("127.0.0.1:8787").await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert!(ready, "wrangler dev did not start");

    let client = reqwest::Client::new();
    let resp = client
        .get("http://127.0.0.1:8787/sse")
        .send()
        .await
        .expect("request failed");

    let mut stream = resp.bytes_stream();
    let first = stream
        .next()
        .await
        .expect("no event received")
        .expect("stream error");
    let body = String::from_utf8(first.to_vec()).expect("utf8");
    assert!(body.contains("Hello from the edge"));

    child.kill().expect("failed to stop wrangler");
}
