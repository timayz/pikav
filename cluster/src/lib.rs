use bytes::Bytes;
use pikav::{
    publisher::{Message, Publisher},
    topic::{TopicFilter, TopicName},
    Event,
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
    pub publisher: Publisher<Bytes>,
    pub nodes: Vec<Client>,
}

#[tonic::async_trait]
impl pikav_client::timada::pikav_server::Pikav for Pikav {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishReply>, Status> {
        let req = request.into_inner();

        let mut messages: Vec<Message<Value, Value>> = Vec::new();

        for e in req.events.iter() {
            let topic = match TopicName::new(e.topic.to_owned()) {
                Ok(name) => name,
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            };

            messages.push(Message {
                event: Event {
                    topic,
                    name: e.name.to_owned(),
                    data: e.data.clone().into(),
                    metadata: e.metadata.clone().map(Into::into),
                },
                user_id: e.user_id.to_owned(),
            });
        }

        self.publisher.publish(messages.iter().collect::<_>()).await;

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

        self.publisher
            .subscribe(filter, req.user_id, req.client_id)
            .await
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

        self.publisher
            .unsubscribe(filter, req.user_id, req.client_id)
            .await
            .ok();

        Ok(Response::new(UnsubscribeReply { success: true }))
    }
}

pub struct ClusterOptions {
    pub addr: String,
    pub publisher: Publisher<Bytes>,
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
            publisher: self.options.publisher.clone(),
            nodes: self.options.nodes.clone(),
        };

        println!("PikavServer listening on {addr}");

        Server::builder()
            .add_service(PikavServer::new(pikav))
            .serve(addr)
            .await
    }
}
