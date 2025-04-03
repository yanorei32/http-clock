use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ConnectionCounter {
    count: Arc<AtomicUsize>,
}

pub struct Session<'a> {
    counter: &'a ConnectionCounter,
}

impl ConnectionCounter {
    pub fn new() -> Self {
        ConnectionCounter {
            count: Arc::new(AtomicUsize::from(0)),
        }
    }

    pub fn acquire(&self) -> Session {
        self.count.fetch_add(1, Ordering::Relaxed);
        tracing::info!("Connection is established. conns: {}", self.current());

        Session { counter: self }
    }

    pub fn release(&self) {
        self.count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn current(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        self.counter.release();
        tracing::info!(
            "Connection has been closed. conns: {}",
            self.counter.current()
        );
    }
}
