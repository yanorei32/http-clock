use crate::{model::Context, Clock, ConnectionCounter};

use async_stream::try_stream;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName},
    response::IntoResponse,
};
use bytes::Bytes;
use futures::Stream;

pub fn encode(ctx: &Context) -> Bytes {
    let jst = ctx.jst.as_str();
    let connection_count = ctx.connection_count;

    bytes::Bytes::from(format!(
        "<option selected>{jst} (JST) / {connection_count} active connection(s).</option>\n"
    ))
}

fn stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<Bytes, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield Bytes::from_static(include_bytes!("../assets/select_head.html"));
        clock.mark_unchanged();

        loop {
            let _ = clock.changed().await;
            let partial = clock.borrow_and_update().select.clone();
            yield partial;
        }
    }
}

pub async fn handler(
    headers: HeaderMap,
    State((clock, counter)): State<(Clock, ConnectionCounter)>,
) -> impl IntoResponse {
    let stream = stream(clock, counter);
    let body = Body::from_stream(stream);

    let is_cloudflare = headers.contains_key("cf-ray");

    let headers = [
        (
            header::CONTENT_TYPE,
            if is_cloudflare {
                "application/grpc"
            } else {
                "text/html; charset=utf-8"
            },
        ),
        (
            HeaderName::from_static("x-original-content-type"),
            "text/html; charset=utf-8",
        ),
    ];

    (headers, body)
}
