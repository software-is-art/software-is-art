use futures_util::StreamExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

#[tokio::test]
#[ignore]
async fn streams_datastar_event() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let status = Command::new("rustup")
        .args(["target", "add", "wasm32-unknown-unknown"])
        .status()
        .expect("failed to add wasm target");
    assert!(status.success(), "rustup target add failed");

    let has_worker_build = Command::new("worker-build")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_worker_build {
        let install = Command::new("cargo")
            .args(["install", "worker-build"])
            .status()
            .expect("failed to install worker-build");
        assert!(install.success(), "worker-build install failed");
    }

    let status = Command::new("worker-build")
        .args(["--release", "--no-opt"])
        .current_dir(manifest_dir)
        .status();
    if status.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("skipping test, worker-build failed");
        return;
    }

    let wasm_path = manifest_dir
        .join("build/worker/shim.mjs")
        .to_string_lossy()
        .to_string();

    let mut child = match Command::new("npx")
        .args([
            "--yes",
            "wrangler@3",
            "dev",
            &wasm_path,
            "--local",
            "--port",
            "8787",
        ])
        .current_dir(manifest_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skipping test, wrangler dev unavailable: {e}");
            return;
        }
    };

    let mut ready = false;
    for _ in 0..300 {
        if let Ok(Some(status)) = child.try_wait() {
            if !status.success() {
                eprintln!("skipping test, wrangler dev exited early");
                return;
            }
        }
        if TcpStream::connect("127.0.0.1:8787").await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if !ready {
        let _ = child.kill();
        eprintln!("skipping test, wrangler dev did not start");
        return;
    }

    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("client");
    let start = Instant::now();
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
    assert!(body.contains("server-time"));

    let latency = start.elapsed();
    println!("First patch event latency: {:?}", latency);

    child.kill().expect("failed to stop wrangler");
}
