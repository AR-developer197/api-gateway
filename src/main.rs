use std::{convert::Infallible, net::SocketAddr};

use hyper::{body::{Bytes, Incoming}, server::conn::http1, Request, Response};
use http_body_util::Full;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::Service;

type Req = Request<Incoming>;

#[derive(Clone, Debug)]
struct Logger<S> {
    inner: S
}

impl<S> Logger<S> {
    fn new(inner: S) -> Self{
        Self {inner}
    }
}

impl <S> Service<Req> for Logger<S> 
where 
    S: Service<Req> + Clone
{
    type Response = S::Response;

    type Error =  S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.inner.call(req)
    }
}


async fn backend_service(_req: Req) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("backend_service_1"))))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let connection = TcpListener::bind(addr).await?;

    loop {
        let (socket, _) = connection.accept().await?;
        let io = TokioIo::new(socket);

        tokio::spawn(async move{
            let http = http1::Builder::new();

            

            let sve = tower::service_fn(backend_service);

            let sve = tower::builder::ServiceBuilder::new().layer_fn(Logger::new).service(sve);
        

            let sve = TowerToHyperService::new(sve);

            if let Err(err) = http.serve_connection(io, sve).await {
                println!("Server Error: {}", err)
            }
        });
    }
}
