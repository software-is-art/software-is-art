use async_stream::stream;
use datastar::{prelude::*, DatastarEvent};
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    if req.path().starts_with("/sse") {
        let resp_stream = stream! {
            let patch = PatchElements::new("<div id='message'>Hello from the edge!</div>");
            let event: DatastarEvent = patch.as_datastar_event();
            yield Ok::<Vec<u8>, Error>(event.to_string().into_bytes());
        };

        let headers = Headers::new();
        headers.set("content-type", "text/event-stream")?;
        headers.set("cache-control", "no-cache")?;
        headers.set("connection", "keep-alive")?;

        let response = Response::from_stream(resp_stream)?.with_headers(headers);
        Ok(response)
    } else {
        Response::ok("datastar edge worker")
    }
}
