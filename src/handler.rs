use crate::{Clock, ConnectionCounter};

use async_stream::try_stream;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName},
    response::IntoResponse,
};
use bytes::Bytes;
use futures::Stream;

fn create_html_stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<Bytes, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield Bytes::from_static(include_bytes!("../assets/head.html"));
        clock.mark_unchanged();

        loop {
            let _ = clock.changed().await;
            let partial_html = clock.borrow_and_update().partial_html.clone();
            yield partial_html;
        }
    }
}

pub async fn handler(
    headers: HeaderMap,
    State((clock, counter)): State<(Clock, ConnectionCounter)>,
) -> impl IntoResponse {
    let stream = create_html_stream(clock, counter);
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
