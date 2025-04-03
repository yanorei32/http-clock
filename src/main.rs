use std::net::SocketAddr;

use axum::{routing::get, Router};
use chrono::{DateTime, Utc};
use chrono_tz::Japan;
use clap::Parser;
use tokio::{net::TcpListener, sync::watch};

mod connection_counter;
mod gif_banner;
mod html;
mod model;
mod mygif;
mod select;
mod svg;

use connection_counter::ConnectionCounter;

type Clock = watch::Receiver<model::ClockData>;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env)]
    #[clap(default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

fn encode(previous_timestamp: i64, connection_count: usize) -> (i64, model::ClockData) {
    let utc: DateTime<Utc> = Utc::now();
    let utc = utc
        .checked_add_signed(chrono::TimeDelta::new(1, 0).unwrap())
        .unwrap();

    let timestamp = utc.timestamp_millis();

    let jst = utc
        .with_timezone(&Japan)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let ctx = model::Context {
        previous_timestamp,
        timestamp,
        connection_count,
        jst,
    };

    (
        timestamp,
        model::ClockData {
            svg: svg::encode(&ctx),
            html: html::encode(&ctx),
            select: select::encode(&ctx),
            gif: gif_banner::encode(&ctx),
        },
    )
}

#[tokio::main]
async fn main() {
    let c = Cli::parse();

    gif_banner::initialization();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let connection_counter = ConnectionCounter::new();

    let (clock_source, clock) = watch::channel(encode(0, 0).1);

    let app = Router::new()
        .route("/", get(html::handler))
        .route("/banner.gif", get(gif_banner::handler))
        .route("/svg", get(svg::handler))
        .route("/select", get(select::handler))
        .with_state((clock, connection_counter.clone()));

    tokio::spawn(async move {
        let mut previous_timestamp: i64 = 0;

        loop {
            let utc: DateTime<Utc> = Utc::now();
            let time = encode(previous_timestamp, connection_counter.current());
            let differencial = 1000 - utc.timestamp_subsec_millis();

            tokio::time::sleep(std::time::Duration::from_millis(differencial as u64)).await;

            clock_source.send(time.1).unwrap();
            previous_timestamp = time.0;
        }
    });

    let listener = TcpListener::bind(c.listen).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
