use crate::{Clock, ConnectionCounter};
use async_stream::try_stream;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName},
    response::IntoResponse,
};
use futures::Stream;

fn create_html_stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<String, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield include_str!("../assets/head.html").to_string();

        let mut event_count = 1;
        loop {
            let time = clock.borrow_and_update().clone();
            let connection_count = counter.current();
            let user_emojis: String = std::iter::repeat("👤").take(connection_count).collect();
            let jst_s = time.0;
            let timestamp = time.1;

            yield format!("\
    <div class=t{timestamp}>\
    <h2>{jst_s} <small>(JST)</small></h2>\
    <p>{event_count} event(s) sent.</p>\
    <p>{connection_count} active connection(s).</p>\
    <p>{user_emojis}</p>\
    </div>");

            let _ = clock.changed().await;

            event_count += 1;
            yield format!("<style>.t{timestamp} {{ display: none; }}</style>\n");
        }
    }
}

pub async fn handler(
    headers: HeaderMap,
    State((clock, counter)): State<(Clock, ConnectionCounter)>,
) -> impl IntoResponse {
    let stream = create_html_stream(clock, counter);
    let body = Body::from_stream(stream);

    let is_cloudflare = headers.contains_key("cf-request-id");

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
