use crate::scheduler::Scheduler;
use sqlx::{Pool, Postgres};

use prometheus::{opts, IntGaugeVec};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;

use std::pin::Pin;
use std::future::{Future};
use std::task::{Context, Poll};
use futures::future::{ok, Ready};

use crate::health::pod::PodInfo;
use actix_web::http::{HeaderName, HeaderValue};
use actix_web_prom::{PrometheusMetrics};

pub struct HealthCounters {
    pod: PodInfo,
    idle_db_conn: IntGaugeVec,
    opened_db_conn: IntGaugeVec,
    current_requests: IntGaugeVec,
}
impl HealthCounters {
    pub fn new() -> Self {
        let idle_opts = opts!("idle_db_conn", "Amount of idle connections").namespace("api");
        let opened_opts = opts!("open_db_conn", "Amount of open connections").namespace("api");
        let current_opts = opts!("current_requests", "Currently open requests").namespace("api");
        Self {
            pod: PodInfo::new(),
            idle_db_conn: IntGaugeVec::new(idle_opts, &[]).unwrap(),
            opened_db_conn: IntGaugeVec::new(opened_opts, &[]).unwrap(),
            current_requests: IntGaugeVec::new(current_opts, &["endpoint", "method"]).unwrap(),
        }
    }
    pub fn register(&self, builder: &mut PrometheusMetrics) {
        builder
            .registry
            .register(Box::new(self.opened_db_conn.clone())).unwrap();
        builder
            .registry
            .register(Box::new(self.idle_db_conn.clone())).unwrap();
        builder
            .registry
            .register(Box::new(self.current_requests.clone())).unwrap();
    }
    pub fn schedule(&self, pool: Pool<Postgres>, scheduler: &mut Scheduler) {
        let this = self.clone();
        scheduler.run(std::time::Duration::from_secs(5), move || {
            let idle = pool.num_idle();
            let total = pool.size();
            this.idle_db_conn.with_label_values(&[]).set(idle as i64);
            this.opened_db_conn.with_label_values(&[]).set(total as i64);
            async move {
                ok::<i32, i32>(1).await.unwrap();
            }
        });
    }
}

impl Clone for HealthCounters {
    fn clone(&self) -> Self {
        Self {
            pod: self.pod.clone(),
            idle_db_conn: self.idle_db_conn.clone(),
            opened_db_conn: self.opened_db_conn.clone(),
            current_requests: self.current_requests.clone(),
        }
    }
}

impl<S, B> Transform<S> for HealthCounters
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MonitoringMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MonitoringMiddleware { service, counters: self.clone() })
    }
}




pub struct MonitoringMiddleware<S> {
    service: S,
    counters: HealthCounters,
}

impl<S, B> Service for MonitoringMiddleware<S>
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // The request has started.
        let pattern_or_path = req.match_pattern().unwrap_or("unknown".to_string());
        let counter = self.counters.current_requests.with_label_values(&[&*pattern_or_path,req.method().as_str()]);
        counter.inc();
        let pod = self.counters.pod.clone();
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res: Self::Response = fut.await?;
            // The request finished, remove a counter
            counter.dec();
            res.headers_mut().insert(HeaderName::from_static("x-server"), HeaderValue::from_str(&*pod.get_id()).unwrap());
            Ok(res)
        })
    }
}