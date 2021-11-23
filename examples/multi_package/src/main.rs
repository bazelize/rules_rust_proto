use service_proto::google;
use service_proto::grpc::examples::echo::{echo_server, EchoRequest};

use std::net::ToSocketAddrs;
use tonic::{transport::Server, Request, Response, Status};

type EchoResult<T> = Result<Response<T>, Status>;
use async_trait::async_trait;

#[derive(Debug)]
pub struct EchoServer {}

#[async_trait]
impl echo_server::Echo for EchoServer {
    async fn unary_echo(&self, _: Request<EchoRequest>) -> EchoResult<google::protobuf::Empty> {
        Err(Status::unimplemented("not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = EchoServer {};
    Server::builder()
        .add_service(echo_server::EchoServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
