use actix_rt::Arbiter;
use futures::StreamExt;

pub struct Scheduler {
    arbiter: Arbiter,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            arbiter: Arbiter::new(),
        }
    }

    pub fn run<F, R>(&mut self, interval: std::time::Duration, mut task: F)
    where
        F: FnMut() -> R + Send + 'static,
        R: std::future::Future<Output = ()> + Send + 'static,
    {
        let future = IntervalStream::new(actix_rt::time::interval(interval))
            .for_each_concurrent(2, move |_| task());

        self.arbiter.spawn(future);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.arbiter.stop();
    }
}

use tokio_stream::wrappers::IntervalStream;
