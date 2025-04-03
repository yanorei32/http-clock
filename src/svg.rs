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
    let timestamp = ctx.timestamp;
    let connection_count = ctx.connection_count;

    let user_emojis: String = if connection_count <= 20 {
        "👤".repeat(connection_count)
    } else {
        "👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤..".to_string()
    };

    bytes::Bytes::from(format!("
    <rect x=\"0\" y=\"0\" width=\"320\" height=\"120\" fill=\"black\" />
    <text font-size=\"2em\" x=\"160\" y=\"40\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"white\">{jst}</text>
    <defs>
        <clipPath id=\"clip{timestamp}\">
            <text font-size=\"0.5em\" x=\"160\" y=\"80\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"white\">\
                Conns: {user_emojis}\
            </text>
        </clipPath>
    </defs>
    <rect x=\"0\" y=\"0\" width=\"320\" height=\"120\" fill=\"white\" clip-path=\"url(#clip{timestamp})\"/>
    "))
}

fn stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<Bytes, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield Bytes::from_static(include_bytes!("../assets/svg_head.html"));
        clock.mark_unchanged();

        loop {
            let _ = clock.changed().await;
            let partial = clock.borrow_and_update().svg.clone();
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
