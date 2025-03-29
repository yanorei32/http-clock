use std::net::SocketAddr;

use axum::{routing::get, Router};
use chrono::{DateTime, Utc};
use chrono_tz::Japan;
use clap::Parser;
use tokio::{net::TcpListener, sync::watch};

mod connection_counter;
mod handler;
mod select_handler;
mod svg_handler;

use connection_counter::ConnectionCounter;
use handler::handler;
use select_handler::select_handler;
use svg_handler::svg_handler;

type Clock = watch::Receiver<ClockData>;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env)]
    #[clap(default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

#[derive(Debug, Clone)]
struct ClockData {
    pub partial_html: bytes::Bytes,
    pub partial_svg: bytes::Bytes,
    pub partial_select: bytes::Bytes,
    pub timestamp: i64,
}

struct Context {
    previous_timestamp: i64,
    timestamp: i64,
    connection_count: usize,
    jst: String,
}

impl Context {
    fn encode_svg(&self) -> bytes::Bytes {
        let jst = self.jst.as_str();
        let timestamp = self.timestamp;
        let connection_count = self.connection_count;

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

    fn encode_html(&self) -> bytes::Bytes {
        let jst = self.jst.as_str();
        let timestamp = self.timestamp;
        let connection_count = self.connection_count;
        let previous_timestamp = self.previous_timestamp;

        let user_emojis: String = if connection_count <= 50 {
            "👤".repeat(connection_count)
        } else {
            "👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤👤..".to_string()
        };

        bytes::Bytes::from(format!(
            "
            <style>.e{previous_timestamp} {{ display: none; }}</style>\
            <div class=e{timestamp}>\
                <h2>{jst} <small>(JST)</small></h2>\
                <p>{connection_count} active connection(s).</p>\
                <p>{user_emojis}</p>\
            </div>\
        "
        ))
    }

    fn encode_select(&self) -> bytes::Bytes {
        let jst = self.jst.as_str();
        let connection_count = self.connection_count;

        bytes::Bytes::from(format!(
            "<option selected>{jst} (JST) / {connection_count} active connection(s).</option>\n"
        ))
    }
}

fn encode(previous_timestamp: i64, connection_count: usize) -> ClockData {
    let utc: DateTime<Utc> = Utc::now();
    let utc = utc
        .checked_add_signed(chrono::TimeDelta::new(1, 0).unwrap())
        .unwrap();

    let timestamp = utc.timestamp_millis();

    let jst = utc
        .with_timezone(&Japan)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let ctx = Context {
        previous_timestamp,
        timestamp,
        connection_count,
        jst,
    };

    let partial_svg = ctx.encode_svg();
    let partial_html = ctx.encode_html();
    let partial_select = ctx.encode_select();

    ClockData {
        partial_svg,
        partial_html,
        partial_select,
        timestamp,
    }
}

#[tokio::main]
async fn main() {
    let c = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let connection_counter = ConnectionCounter::new();

    let (clock_source, clock) = watch::channel(encode(0, 0));

    let app = Router::new()
        .route("/", get(handler))
        .route("/svg", get(svg_handler))
        .route("/select", get(select_handler))
        .with_state((clock, connection_counter.clone()));

    tokio::spawn(async move {
        let mut previous_timestamp: i64 = 0;

        loop {
            let utc: DateTime<Utc> = Utc::now();
            let time = encode(previous_timestamp, connection_counter.current());
            let differencial = 1000 - utc.timestamp_subsec_millis();

            tokio::time::sleep(std::time::Duration::from_millis(differencial as u64)).await;

            clock_source.send(time.clone()).unwrap();
            previous_timestamp = time.timestamp;
        }
    });

    let listener = TcpListener::bind(c.listen).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
