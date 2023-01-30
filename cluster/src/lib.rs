use bytes::Bytes;
use pikav::{topic::TopicName, Event, PubEvent};
use pikav_client::Client;
use timada::pikav_server::{Pikav as PikavRpc, PikavServer};
use timada::{
    PublishReply, PublishRequest, SubscribeReply, SubscribeRequest, UnsubscribeReply,
    UnsubscribeRequest,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::error;

mod timada {
    tonic::include_proto!("timada");
}

#[derive(Default)]
pub struct Pikav {
    pub pikav: pikav::Pikav<Bytes>,
    pub nodes: Vec<Client>,
}

#[tonic::async_trait]
impl PikavRpc for Pikav {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let req = request.into_inner();
        let mut pub_events = Vec::new();

        for e in req.events.iter() {
            let topic = match TopicName::new(e.topic.to_owned()) {
                Ok(name) => name,
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            };

            pub_events.push(PubEvent {
                event: Event {
                    topic,
                    name: e.name.to_owned(),
                    data: "None",
                    metadata: "None", // data: e.data.clone(),
                                      // metadata: e.metadata.clone(),
                },
                user_id: format!("{}/{}", req.namespace, e.user_id),
            });
        }

        self.pikav.publish(pub_events.iter().collect::<_>());

        if req.propagate {
            // for node in self.nodes.iter() {
            //     let mut client = PikavClient::new(node.clone());
            //     let mut req = req.clone();
            //     req.propagate = false;

            //     if let Err(e) = client.publish(req).await {
            //         error!("{e}");
            //     }
            // }
        }

        Ok(Response::new(PublishReply { success: true }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        Ok(Response::new(SubscribeReply { success: true }))
    }

    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        Ok(Response::new(UnsubscribeReply { success: true }))
    }
}

pub struct ClusterOptions {
    pub addr: String,
    pub pikav: pikav::Pikav<Bytes>,
    pub nodes: Vec<Client>,
}

pub struct Cluster {
    pub options: ClusterOptions,
}

impl Cluster {
    pub fn new(options: ClusterOptions) -> Self {
        Self { options }
    }

    pub async fn serve(&self) -> Result<(), tonic::transport::Error> {
        let addr = self.options.addr.parse().unwrap();

        let pikav = Pikav {
            pikav: self.options.pikav.clone(),
            nodes: self.options.nodes.clone(),
        };

        println!("PikavServer listening on {}", addr);

        Server::builder()
            .add_service(PikavServer::new(pikav))
            .serve(addr)
            .await
    }
}
