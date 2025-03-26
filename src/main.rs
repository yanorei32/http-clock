use std::net::SocketAddr;

use axum::{routing::get, Router};
use chrono::{DateTime, Utc};
use chrono_tz::Japan;
use tokio::{net::TcpListener, sync::watch};
use clap::Parser;

mod connection_counter;
mod handler;
mod svg_handler;

use connection_counter::ConnectionCounter;
use handler::handler;
use svg_handler::svg_handler;

type Time = (String, i64);
type Clock = watch::Receiver<Time>;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env)]
    #[clap(default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

fn current_time() -> Time {
    let utc: DateTime<Utc> = Utc::now();
    let timestamp = utc.timestamp_millis();
    let jst = utc.with_timezone(&Japan);

    (jst.format("%Y-%m-%d %H:%M:%S").to_string(), timestamp)
}

#[tokio::main]
async fn main() {
    let c = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (clock_source, clock) = watch::channel(current_time());

    let app = Router::new()
        .route("/", get(handler))
        .route("/svg", get(svg_handler))
        .with_state((clock, ConnectionCounter::new()));

    tokio::spawn(async move {
        loop {
            let utc: DateTime<Utc> = Utc::now();
            let differencial = 1000 - utc.timestamp_subsec_millis();

            tokio::time::sleep(std::time::Duration::from_millis(differencial as u64)).await;
            clock_source.send(current_time()).unwrap();
        }
    });

    let listener = TcpListener::bind(c.listen).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
