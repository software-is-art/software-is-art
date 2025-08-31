# Software is Art Monorepo

This repository hosts experimental Rust projects built with a Buck2 based workflow.
Each micro crate lives under `crates/` and can be cached and deployed independently.

## Crates

- **datastar-edge-worker** – Server‑Sent Events example using [Datastar](https://crates.io/crates/datastar) on
  [Cloudflare Workers](https://crates.io/crates/worker).

  Run `wrangler dev` and open the root page to see a demo with the browser clock
  and a server clock patched in real time via Datastar events. The page
  imports the Datastar client library and applies incoming patch events to update
  the server clock.

## CI

Buck2 orchestration lives in `tools/ci.bxl` and can be executed with
`buck2 bxl //tools:ci.bxl` once the toolchain and third‑party deps are configured.

## Tests

The `datastar-edge-worker` crate includes an end‑to‑end test that boots the worker with `wrangler dev` and verifies the streamed event. The test also records how long it takes for the first Datastar patch event to arrive, providing a simple view into edge worker latency. Run it with:

```bash
cargo test -p datastar-edge-worker -- --ignored
```

### Wrangler dev

Compile the worker to WebAssembly with `worker-build` and launch `wrangler dev`:

```bash
rustup target add wasm32-unknown-unknown
cargo install worker-build
worker-build --release
npx wrangler dev build/worker/shim.mjs --local
```

If Buck2 is available it can cache the build artifacts:

```bash
buck2 build //crates/infra/datastar-edge-worker:datastar_edge_worker --target-platform=wasm32-unknown-unknown
wrangler dev $(buck2 targets --show-output //crates/infra/datastar-edge-worker:datastar_edge_worker --target-platform=wasm32-unknown-unknown | cut -d' ' -f2) --local
```

`buck2` caches the wasm output so `wrangler dev` only rebuilds when the source or configuration changes.
