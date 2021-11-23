use echo_proto::grpc::examples::echo::{echo_server, EchoRequest, EchoResponse};

use futures::Stream;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use tonic::{transport::Server, Request, Response, Status, Streaming};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<EchoResponse, Status>> + Send>>;
use async_trait::async_trait;

#[derive(Debug)]
pub struct EchoServer {}

#[async_trait]
impl echo_server::Echo for EchoServer {
    async fn unary_echo(&self, _: Request<EchoRequest>) -> EchoResult<EchoResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    type ServerStreamingEchoStream = ResponseStream;

    async fn server_streaming_echo(
        &self,
        req: Request<EchoRequest>,
    ) -> EchoResult<Self::ServerStreamingEchoStream> {
        println!("Client connected from: {:?}", req.remote_addr());

        let (tx, rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let _ = rx.await;
            println!("The rx resolved therefore the client disconnected!");
        });

        struct ClientDisconnect(oneshot::Sender<()>);

        impl Stream for ClientDisconnect {
            type Item = Result<EchoResponse, Status>;

            fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                // A stream that never resovlves to anything....
                Poll::Pending
            }
        }

        Ok(Response::new(
            Box::pin(ClientDisconnect(tx)) as Self::ServerStreamingEchoStream
        ))
    }

    async fn client_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<EchoResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<Self::BidirectionalStreamingEchoStream> {
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
