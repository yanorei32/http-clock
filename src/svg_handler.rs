use crate::{Clock, ConnectionCounter};
use async_stream::try_stream;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName},
    response::IntoResponse,
};
use futures::Stream;

fn create_svg_stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<String, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield include_str!("../assets/svg_head.html").to_string();

        let mut event_count = 1;
        loop {
            let time = clock.borrow_and_update().clone();
            let jst_s = time.0;
            let connection_count = counter.current();
            let user_emojis: String = "👤".repeat(connection_count);

            yield format!("
                <rect x=\"0\" y=\"0\" width=\"320\" height=\"120\" fill=\"black\" />
                <text font-size=\"2em\" x=\"160\" y=\"40\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"white\">{jst_s}</text>
                <defs>
                    <clipPath id=\"clip{event_count}\">
                        <text font-size=\"0.5em\" x=\"160\" y=\"80\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"white\">Conns: {user_emojis}</text>
                    </clipPath>
                </defs>
                <rect x=\"0\" y=\"0\" width=\"320\" height=\"120\" fill=\"white\" clip-path=\"url(#clip{event_count})\"/>
            ");

            let _ = clock.changed().await;
            event_count += 1;
        }
    }
}


pub async fn svg_handler(
    headers: HeaderMap,
    State((clock, counter)): State<(Clock, ConnectionCounter)>,
) -> impl IntoResponse {
    let stream = create_svg_stream(clock, counter);
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
