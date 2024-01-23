use json::JsonValue;
use log::info;
use mongodb::Collection;
use tonic::{Request, Response, Status};

use bson::Uuid;
use dapr::{appcallback::*, dapr::dapr::proto::runtime::v1::app_callback_server::AppCallback};

use crate::foreign_types::ProductVariant;

pub struct AppCallbackService {
    pub collection: Collection<ProductVariant>,
}

impl AppCallbackService {
    /// Add a newly created product variant to MongoDB.
    pub async fn add_product_variant_to_mongodb(
        &self,
        product_variant_id: Uuid,
    ) -> Result<(), Status> {
        let product_variant = ProductVariant {
            _id: product_variant_id,
        };
        match self.collection.insert_one(product_variant, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Status::internal(
                "Adding product variant failed in MongoDB.",
            )),
        }
    }
}

#[tonic::async_trait]
impl AppCallback for AppCallbackService {
    /// Invokes service method with InvokeRequest.
    async fn on_invoke(
        &self,
        _request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        Ok(Response::new(InvokeResponse::default()))
    }

    /// Lists all topics subscribed by this app.
    ///
    /// NOTE: Dapr runtime will call this method to get
    /// the list of topics the app wants to subscribe to.
    /// In this example, the app is subscribing to topic `A`.
    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        let topic = "catalog/product-variant/created".to_string();
        let pubsub_name = "pubsub".to_string();

        let list_subscriptions = ListTopicSubscriptionsResponse::topic(pubsub_name, topic);

        Ok(Response::new(list_subscriptions))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        let r = request.into_inner();
        let data = &r.data;

        let message = String::from_utf8_lossy(data);
        let error_message = format!("Expected message to be parsable JSON, got: {}", message);
        let message_json = json::parse(&message).map_err(|_| Status::internal(error_message))?;
        let product_variant_id_json_value = &message_json["id"];
        let product_variant_id = parse_product_variant_id(product_variant_id_json_value)?;

        info!("Event with message was received: {}", &message);

        self.add_product_variant_to_mongodb(product_variant_id)
            .await?;

        Ok(Response::new(TopicEventResponse::default()))
    }

    /// Lists all input bindings subscribed by this app.
    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        Ok(Response::new(ListInputBindingsResponse::default()))
    }

    /// Listens events from the input bindings.
    async fn on_binding_event(
        &self,
        _request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        Ok(Response::new(BindingEventResponse::default()))
    }
}

/// Parses Uuid from JsonValue containing a String.
fn parse_product_variant_id(product_variant_id_json_value: &JsonValue) -> Result<Uuid, Status> {
    match product_variant_id_json_value {
        json::JsonValue::String(product_variant_id_string) => {
            match Uuid::parse_str(product_variant_id_string) {
                Ok(product_variant_id_uuid) => Ok(product_variant_id_uuid),
                Err(_) => {
                    let error_message = format!(
                        "String value in `id` field cannot be parsed as bson::Uuid, got: {}",
                        product_variant_id_string
                    );
                    Err(Status::internal(error_message))?
                }
            }
        }
        _ => {
            let error_message = format!(
                "`id` field does not exist or does not contain a String value, got: {}",
                product_variant_id_json_value
            );
            Err(Status::internal(error_message))?
        }
    }
}
