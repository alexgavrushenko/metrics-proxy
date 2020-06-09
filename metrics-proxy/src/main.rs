mod config;
mod proxy;
mod stats;

#[allow(unused_imports)]
/// Import longer-name versions of macros only to not collide with legacy `log`
#[macro_use(slog_error, slog_info, slog_warn, slog_o)]
extern crate slog;

use futures::{pin_mut, select, FutureExt};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use log::info;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        // This will spawn into the currently running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    block_on( async move {
        stats::init();    
        let stats_adress: std::net::SocketAddr = "0.0.0.0:9102".parse().unwrap();
        let stats_exporter =
                run_stats_exporter(&stats_adress).fuse();
    
        let http_server = run().fuse();
    
        pin_mut!(http_server, stats_exporter);
        select! {
            r = http_server => r,
            r = stats_exporter => r,
        }
    })
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app = clap::App::new("metrics-proxy");
    let app_config = config::from_args_proxy_config(app);
    let config = Rc::new(app_config.service_config);
    let conf = config.clone();

    let new_service = make_service_fn(move |_: &AddrStream| {
        let conf = conf.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let conf = conf.clone();
                async move { proxy::route(req, &conf).await }
            }))
        }
    });

    info!("Listen on {}", &config.listener_addr);
    hyper::Server::try_bind(&config.listener_addr)?
        .executor(LocalExec)
        .serve(new_service)
        .await?;
    Ok(())
}

thread_local! {
    static PANIC_SENDER:
        std::cell::Cell<Option<tokio::sync::oneshot::Sender<Box<dyn std::any::Any + Send>>>>
            = std::cell::Cell::new(None);
}

pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    let local = tokio::task::LocalSet::new();

    let (tx, rx) = tokio::sync::oneshot::channel();
    PANIC_SENDER.with(|s| s.set(Some(tx)));

    let task = async move {
        select! {
            res = future.fuse() => return res,
            res = rx.fuse() => std::panic::resume_unwind(res.unwrap()),
        };
    };

    local.block_on(&mut rt, task)
}

/// Returns an HTTP server bound to the address specified. The server exposes metrics in Prometheus
/// format on the path `/metrics`. It also shows a help message on the default path `/`.
///
/// To generate a report, the exporter calls the `prometheus`'s crate `gather()`. Hence, it only
/// exposes metrics from the default crate's registry.
async fn run_stats_exporter(addr: &std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    use hyper::{Body, Request, Response, StatusCode};
    use prometheus::{Encoder, TextEncoder};
    use std::convert::Infallible;

    const HELP_MESSAGE: &str = r#"<html>
       <head><title>Prometheus Exporter</title></head>
       <body>
       <h1>Prometheus Exporter</h1>
       <p><a href="/metrics">Metrics</a></p>
       </body>
       </html>"#;

    log::info!("starting prometheus metrics exporter on {}", addr);

    let service = service_fn(|req: Request<Body>| async move {
        let response = match req.uri().path() {
            "/metrics" => {
                let mut buf = Vec::new();
                TextEncoder::new()
                    .encode(&prometheus::gather(), &mut buf)
                    .unwrap_or_else(|err| log::debug!("cannot encode prometheus metrics: {}", err));
                Response::new(Body::from(buf))
            }
            "/" => Response::new(Body::from(HELP_MESSAGE)),
            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        };
        Ok::<_, Infallible>(response)
    });

    let make_svc = make_service_fn(|_| async move { Ok::<_, Infallible>(service) });

    hyper::Server::try_bind(addr)?.serve(make_svc).await?;
    Ok(())
}
