use bytes::Bytes;
use pikav::{
    publisher::{Message, Publisher},
    Event, SimpleEvent,
};
use pikav_client::{
    timada::{
        pikav_server::{self, PikavServer},
        PublishEventsReply, PublishEventsRequest, PublishReply, PublishRequest, SubscribeReply,
        SubscribeRequest, UnsubscribeReply, UnsubscribeRequest,
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
impl pikav_server::Pikav for Pikav {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishReply>, Status> {
        let req = request.into_inner();

        // let mut messages: Vec<Message<SimpleEvent>> = Vec::new();

        // for e in req.events.iter() {
        //     messages.push(Message {
        //         event: SimpleEvent {
        //             topic: e.topic.to_owned(),
        //             event: e.event.to_owned(),
        //             data: e.data.clone().into(),
        //             metadata: e.metadata.clone().map(Into::into),
        //             filters: None,
        //         },
        //         user_id: e.user_id.to_owned(),
        //     });
        // }

        let messages = req
            .events
            .iter()
            .map(|event| Message {
                event: SimpleEvent {
                    topic: event.topic.to_owned(),
                    event: event.event.to_owned(),
                    data: event.data.to_owned(),
                },
                user_id: event.user_id.to_owned(),
            })
            .collect::<_>();

        self.publisher.publish(messages).await;

        if req.propagate {
            for node in self.nodes.iter() {
                node.publish(req.events.clone());
            }
        }

        Ok(Response::new(PublishReply { success: true }))
    }

    async fn publish_events(
        &self,
        request: Request<PublishEventsRequest>,
    ) -> Result<Response<PublishEventsReply>, Status> {
        let req = request.into_inner();

        let messages = req
            .events
            .iter()
            .map(|event| Message {
                event: Event::<Value, Value> {
                    topic: event.topic.to_owned(),
                    name: event.name.to_owned(),
                    data: event.data.clone().into(),
                    metadata: event.metadata.clone().map(Into::into),
                    filters: None,
                },
                user_id: event.user_id.to_owned(),
            })
            .collect::<_>();

        self.publisher.publish_events(messages).await;

        if req.propagate {
            for node in self.nodes.iter() {
                node.publish_events(req.events.clone());
            }
        }

        Ok(Response::new(PublishEventsReply { success: true }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeReply>, Status> {
        let req = request.into_inner();

        self.publisher
            .subscribe(req.filter, req.user_id, req.client_id)
            .await
            .ok();

        Ok(Response::new(SubscribeReply { success: true }))
    }

    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeReply>, Status> {
        let req = request.into_inner();

        self.publisher
            .unsubscribe(req.filter, req.user_id, req.client_id)
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
