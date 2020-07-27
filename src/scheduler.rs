use actix_rt::time;
use actix_rt::Arbiter;
use futures::StreamExt;

pub struct Scheduler {
    arbiter: Arbiter,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            arbiter: Arbiter::new(),
        }
    }

    pub fn run<F, R>(&mut self, interval: std::time::Duration, task: F)
    where
        F: Fn() -> R + Send + 'static,
        R: std::future::Future<Output = ()> + Send + 'static,
    {
        let future = time::interval(interval).for_each_concurrent(2, move |_| task());
        self.arbiter.send(future);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.arbiter.stop();
    }
}
