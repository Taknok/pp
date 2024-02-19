use axum::{
  body::Body,
  extract::Request,
  http::{Method, StatusCode},
  response::{IntoResponse, Response},
  Router,
};

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::upgrade::Upgraded;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tower::Service;
use tower::ServiceExt;

use hyper_util::rt::TokioIo;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod pac_utils;
use crate::pac_utils::PAC_UTILS;

mod pac_parser;
use crate::pac_parser::PACParser;

#[tokio::main]
async fn main() {
  tracing_subscriber::registry()
      .with(
          tracing_subscriber::EnvFilter::try_from_default_env()
              .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
      )
      .with(tracing_subscriber::fmt::layer())
      .init();

  let router_svc = Router::new();

  let mut parser = PACParser::new().await;

  let tower_service = tower::service_fn(|req: Request<_>| {
      let router_svc = router_svc.clone();
      let req = req.map(Body::new);

      let url = pac_utils::get_url(&req);
      let host = String::from(req.uri().host().unwrap());
      println!("url: {}", url);
      println!("host: {}", host);
      // let r = parser.find(&url, &host);
      // println!("r2: {}", r);

      async move {
          if req.method() == Method::CONNECT {
              proxy(req).await
          } else {
              router_svc.oneshot(req).await.map_err(|err| match err {})
          }
      }
  });

  let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
      tower_service.clone().call(request)
  });

  let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
  tracing::debug!("listening on {}", addr);

  let listener = TcpListener::bind(addr).await.unwrap();
  loop {
      let (stream, _) = listener.accept().await.unwrap();
      let io = TokioIo::new(stream);
      let hyper_service = hyper_service.clone();
      tokio::task::spawn(async move {
          if let Err(err) = http1::Builder::new()
              .preserve_header_case(true)
              .title_case_headers(true)
              .serve_connection(io, hyper_service)
              .with_upgrades()
              .await
          {
              println!("Failed to serve connection: {:?}", err);
          }
      });
  }
}

async fn proxy(req: Request) -> Result<Response, hyper::Error> {
  tracing::trace!(?req);

  if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
      tokio::task::spawn(async move {
          match hyper::upgrade::on(req).await {
              Ok(upgraded) => {
                  if let Err(e) = tunnel(upgraded, host_addr).await {
                      tracing::warn!("server io error: {}", e);
                  };
              }
              Err(e) => tracing::warn!("upgrade error: {}", e),
          }
      });

      Ok(Response::new(Body::empty()))
  } else {
      tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
      Ok((
          StatusCode::BAD_REQUEST,
          "CONNECT must be to a socket address",
      )
          .into_response())
  }
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
  let mut server = TcpStream::connect(addr).await?;
  let mut upgraded = TokioIo::new(upgraded);

  let (from_client, from_server) =
      tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

  tracing::debug!(
      "client wrote {} bytes and received {} bytes",
      from_client,
      from_server
  );

  Ok(())
}
