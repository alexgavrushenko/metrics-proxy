use crate::config::ProxyServiceConfig;
use hyper::{Body, Request, Response};

use crate::stats;

pub async fn route(
    req: Request<Body>,
    config: &ProxyServiceConfig,
) -> Result<Response<Body>, hyper::Error> {
    // check HTTP method
    log::trace!("Received request: {}, {}", req.method(), req.uri());
    log::trace!("Config: {:?}", config);
    stats::REQUESTS.inc();
    make_ok_response(vec![])
}

fn make_ok_response<E>(body: Vec<u8>) -> Result<Response<Body>, E> {
    log::debug!("Success response: {}", hex::encode(&body));
    Ok(Response::new(Body::from(body)))
}
