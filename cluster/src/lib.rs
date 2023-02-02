use bytes::Bytes;
use pikav::{
    topic::{TopicFilter, TopicName},
    Event, PubEvent, SubscribeOptions,
};
use pikav_client::{
    timada::{
        pikav_server::PikavServer, PublishReply, PublishRequest, SubscribeReply, SubscribeRequest,
        UnsubscribeReply, UnsubscribeRequest,
    },
    Client,
};
use serde_json::Value;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct Pikav {
    pub pikav: pikav::Pikav<Bytes>,
    pub nodes: Vec<Client>,
}

#[tonic::async_trait]
impl pikav_client::timada::pikav_server::Pikav for Pikav {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishReply>, Status> {
        let req = request.into_inner();

        let mut pub_events: Vec<PubEvent<Value, Value>> = Vec::new();

        for e in req.events.iter() {
            let topic = match TopicName::new(e.topic.to_owned()) {
                Ok(name) => name,
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            };

            pub_events.push(PubEvent {
                event: Event {
                    topic,
                    name: e.name.to_owned(),
                    data: e.data.clone().into(),
                    metadata: e.metadata.clone().into(),
                },
                user_id: e.user_id.to_owned(),
            });
        }

        self.pikav.publish(pub_events.iter().collect::<_>());

        if req.propagate {
            for node in self.nodes.iter() {
                node.publish(req.events.clone());
            }
        }

        Ok(Response::new(PublishReply { success: true }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeReply>, Status> {
        let req = request.into_inner();

        let filter = match TopicFilter::new(req.filter) {
            Ok(filter) => filter,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        self.pikav
            .subscribe(SubscribeOptions {
                filter,
                user_id: req.user_id,
                client_id: req.client_id,
            })
            .ok();

        Ok(Response::new(SubscribeReply { success: true }))
    }

    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeReply>, Status> {
        let req = request.into_inner();

        let filter = match TopicFilter::new(req.filter) {
            Ok(filter) => filter,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        self.pikav
            .unsubscribe(SubscribeOptions {
                filter,
                user_id: req.user_id,
                client_id: req.client_id,
            })
            .ok();

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

        println!("PikavServer listening on {addr}");

        Server::builder()
            .add_service(PikavServer::new(pikav))
            .serve(addr)
            .await
    }
}
