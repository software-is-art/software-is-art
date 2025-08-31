use async_stream::stream;
use datastar::{prelude::*, DatastarEvent};
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    if req.path().starts_with("/sse") {
        let resp_stream = stream! {
            loop {
                let now = Date::new_0()
                    .to_locale_time_string("en-US")
                    .as_string()
                    .unwrap_or_default();
                let patch =
                    PatchElements::new(&format!("<div id='server-time' class='time'>{now}</div>"));
                let event: DatastarEvent = patch.as_datastar_event();
                yield Ok::<Vec<u8>, Error>(event.to_string().into_bytes());
                TimeoutFuture::new(1000).await;
            }
        };

        let headers = Headers::new();
        headers.set("content-type", "text/event-stream")?;
        headers.set("cache-control", "no-cache")?;
        headers.set("connection", "keep-alive")?;

        let response = Response::from_stream(resp_stream)?.with_headers(headers);
        Ok(response)
    } else {
        let html = include_str!("../templates/index.html");
        Response::from_html(html)
    }
}
