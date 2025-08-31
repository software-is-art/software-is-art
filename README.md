# Software is Art Monorepo

This repository hosts experimental Rust projects built with a Buck2 based workflow.
Each micro crate lives under `crates/` and can be cached and deployed independently.

## Crates

- **datastar-edge-worker** – Server‑Sent Events example using [Datastar](https://crates.io/crates/datastar) on
  [Cloudflare Workers](https://crates.io/crates/worker).

## CI

Buck2 orchestration lives in `tools/ci.bxl` and can be executed with
`buck2 bxl //tools:ci.bxl` once the toolchain and third‑party deps are configured.

## Tests

The `datastar-edge-worker` crate includes an end‑to‑end test that boots the worker with `wrangler dev` and verifies the streamed event. Run it with:

```bash
cargo test -p datastar-edge-worker
```

### Wrangler dev with Buck2 caching

Build the worker module once with Buck2 and hand the compiled artifact to `wrangler dev`:

```bash
buck2 build //crates/infra/datastar-edge-worker:datastar_edge_worker --target-platform=wasm32-unknown-unknown
wrangler dev $(buck2 targets --show-output //crates/infra/datastar-edge-worker:datastar_edge_worker --target-platform=wasm32-unknown-unknown | cut -d' ' -f2) --local
```

`buck2` caches the wasm output so `wrangler dev` only rebuilds when the source or configuration changes.
