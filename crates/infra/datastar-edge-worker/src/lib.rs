use async_stream::stream;
use datastar::{prelude::*, DatastarEvent};
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::kv::KvStore;
use worker::*;

#[derive(Default, Serialize, Deserialize)]
struct ClientInfo {
    lat: f32,
    lon: f32,
    latency_ms: Option<f64>,
}

#[derive(Deserialize)]
struct LatencyReport {
    id: String,
    latency: f64,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let kv = env.kv("CLIENTS")?;

    if req.path().starts_with("/sse") {
        let id = req
            .url()
            .ok()
            .and_then(|u| {
                u.query_pairs()
                    .find(|(k, _)| k == "id")
                    .map(|(_, v)| v.into_owned())
            })
            .unwrap_or_default();

        if let Some(cf) = req.cf() {
            if let Some((lat, lon)) = cf.coordinates() {
                let info = ClientInfo {
                    lat,
                    lon,
                    latency_ms: None,
                };
                kv.put(&id, serde_json::to_string(&info)?)?
                    .execute()
                    .await?;
            }
        }

        let kv_stream = kv.clone();
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

                let stats_json = {
                    let mut latencies = Vec::new();
                    let mut clients = Vec::new();
                    if let Ok(list) = kv_stream.list().execute().await {
                        for key in list.keys {
                            if let Ok(Some(info)) = kv_stream.get(&key.name).json::<ClientInfo>().await {
                                if let Some(l) = info.latency_ms {
                                    latencies.push(l);
                                    clients.push(json!({
                                        "id": key.name,
                                        "lat": info.lat,
                                        "lon": info.lon,
                                        "latency": l
                                    }));
                                }
                            }
                        }
                    }
                    json!({ "latencies": latencies, "clients": clients })
                };
                let stats_event = format!("event: stats\ndata: {}\n\n", stats_json);
                yield Ok(stats_event.into_bytes());

                TimeoutFuture::new(1000).await;
            }
        };

        let headers = Headers::new();
        headers.set("content-type", "text/event-stream")?;
        headers.set("cache-control", "no-cache")?;
        headers.set("connection", "keep-alive")?;

        let response = Response::from_stream(resp_stream)?.with_headers(headers);
        Ok(response)
    } else if req.path() == "/latency" && req.method() == Method::Post {
        handle_latency(req, kv).await
    } else {
        let html = include_str!("../templates/index.html");
        Response::from_html(html)
    }
}

async fn handle_latency(mut req: Request, kv: KvStore) -> Result<Response> {
    let report: LatencyReport = req.json().await?;
    let mut info = kv
        .get(&report.id)
        .json::<ClientInfo>()
        .await?
        .unwrap_or_default();
    info.latency_ms = Some(report.latency);
    kv.put(&report.id, serde_json::to_string(&info)?)?
        .execute()
        .await?;
    Response::ok("ok")
}
