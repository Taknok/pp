use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Version};
use std::convert::Infallible;
use std::net::SocketAddr;
use url::Url;
use hyper::body::Bytes;
use hyper::server::conn::AddrStream;

mod pac_utils;
use crate::pac_utils::PAC_UTILS;

mod pac_parser;
use crate::pac_parser::PACParser;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[tokio::main]
async fn main() {
  let mut parser = PACParser::new().await;

  // create Proxy
  let addr = SocketAddr::from(([127, 0, 0, 1], 8888));

  // Create a hyper server and define the request handler
/*  let make_svc = make_service_fn(move |_conn| {
    async move {
      Ok::<_, Infallible>(service_fn(move |req| handle_request(req, parser)))
    }
  });*/

  let make_svc = make_service_fn(|socket: &AddrStream| {
    let remote_addr = socket.remote_addr();
    async move {
        Ok::<_, Infallible>(service_fn(move |_: Request<Body>| async move {
            Ok::<_, Infallible>(
                Response::new(Body::from(format!("Hello, {}!", remote_addr)))
            )
        }))
    }
  });

  // Start the server
  let server = Server::bind(&addr).serve(make_svc);
    
  println!("Proxy server started on http://{}", addr);
    
  if let Err(e) = server.await {
    eprintln!("Server error: {}", e);
  }

  let url = String::from("https://google.com/");
  let host = String::from("google.com");
  let r = parser.find(&url, &host);
  println!("r2: {}", r);
}

async fn my_request(req: Request<Body>, remote_addr: SocketAddr) -> Result<hyper::Response<hyper::Body>, Infallible> {
  Ok::<_, Infallible>(
      Response::new(Body::from(format!("Hello ! {}", remote_addr)))
    )
}


/*fn dummy_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
  print_type_of(&req);
  if req.version() == Version::HTTP_11 {
    Ok(Response::new(Full::<Bytes>::from("Hello World")))
  } else {
    // Note: it's usually better to return a Response
    // with an appropriate StatusCode instead of an Err.
    Err("not HTTP/1.1, abort connection")
  }
}*/

/*
fn get_url<T>(req: &hyper::Request<T>) -> Result<String, String>{
  // Get the request URL as a string
  let url_string = req.uri().to_string();
    
  // Parse the URL using the `url` crate
  if let Ok(url) = Url::parse(&url_string) {
    // Get the URL without the path and query
    let base_url = url.origin().ascii_serialization();
        
    println!("Base URL: {}", base_url);
    Ok(base_url)
  } else {
    println!("Invalid URL");
    Err("Invalid URL".to_string())
  }
}

// Function to handle incoming client requests
async fn handle_request(req: Request<Body>, mut parser: PACParser) -> Result<Response<Body>, hyper::Error> {
    let client = Client::new();
 
    let url = get_url(&req).expect("Error getting url from request");
    let host = String::from(req.uri().host().unwrap());
    let r = parser.find(&url, &host);
    println!("r2: {}", r);

    // Modify the request to change the destination
    let mut modified_request = Request::builder()
        .method(req.method().clone())
        .uri(format!("http://127.0.0.1:8080{}", req.uri()))
        .version(req.version());
    {
      let headers = modified_request.headers_mut().unwrap();
      print_type_of(&headers);
/*      for r in req.headers().iter() {
        headers.insert(r);
      }*/
      print_type_of(req.headers());
      // headers.extend(req.headers());
    }
    let modified_request = modified_request.body(req.into_body()).unwrap();
    
    // Send the modified request to the destination server and get the response
    let res = client.request(modified_request).await?;
    
    Ok(res)
}
*/