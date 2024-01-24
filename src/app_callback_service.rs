use json::JsonValue;
use log::info;
use mongodb::Collection;
use tonic::{Request, Response, Status};

use bson::Uuid;
use dapr::{appcallback::*, dapr::dapr::proto::runtime::v1::app_callback_server::AppCallback};

use crate::{foreign_types::ProductVariant, shoppingcart::ShoppingCart, user::User};

pub struct AppCallbackService {
    pub product_variant_collection: Collection<ProductVariant>,
    pub user_collection: Collection<User>,
}

impl AppCallbackService {
    /// Add a newly created product variant to MongoDB.
    pub async fn add_product_variant_to_mongodb(&self, id: Uuid) -> Result<(), Status> {
        let product_variant = ProductVariant { _id: id };
        match self
            .product_variant_collection
            .insert_one(product_variant, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Status::internal(
                "Adding product variant failed in MongoDB.",
            )),
        }
    }

    /// Add a newly created user to MongoDB.
    pub async fn add_user_to_mongodb(&self, id: Uuid) -> Result<(), Status> {
        let user = User {
            _id: id,
            shoppingcart: ShoppingCart::new(),
        };
        match self.user_collection.insert_one(user, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Status::internal("Adding user failed in MongoDB.")),
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
        let product_variant_topic = "catalog/product-variant/created".to_string();
        let user_topic = "user/user/created".to_string();
        let pubsub_name = "pubsub".to_string();
        let user_topic_subscription = TopicSubscription::new(pubsub_name.clone(), user_topic, None);

        let mut list_subscriptions =
            ListTopicSubscriptionsResponse::topic(pubsub_name, product_variant_topic);
        list_subscriptions
            .subscriptions
            .push(user_topic_subscription);

        Ok(Response::new(list_subscriptions))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        let r: dapr::dapr::dapr::proto::runtime::v1::TopicEventRequest = request.into_inner();
        let data = &r.data;

        let message = String::from_utf8_lossy(data);
        let error_message = format!("Expected message to be parsable JSON, got: {}", message);
        let message_json = json::parse(&message).map_err(|_| Status::internal(error_message))?;
        let id_json_value = &message_json["id"];
        let id = parse_id(id_json_value)?;

        info!("Event with message was received: {}", &message);

        match r.topic.as_str() {
            "catalog/product-variant/created" => self.add_product_variant_to_mongodb(id).await?,
            "user/user/created" => self.add_user_to_mongodb(id).await?,
            _ => {
                let message = format!(
                    "Event of topic: `{}` is not a handable by this service.",
                    r.topic.as_str()
                );
                Err(Status::internal(message))?;
            }
        }

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
fn parse_id(id_json_value: &JsonValue) -> Result<Uuid, Status> {
    match id_json_value {
        json::JsonValue::String(id_string) => match Uuid::parse_str(id_string) {
            Ok(id_uuid) => Ok(id_uuid),
            Err(_) => {
                let error_message = format!(
                    "String value in `id` field cannot be parsed as bson::Uuid, got: {}",
                    id_string
                );
                Err(Status::internal(error_message))?
            }
        },
        _ => {
            let error_message = format!(
                "`id` field does not exist or does not contain a String value, got: {}",
                id_json_value
            );
            Err(Status::internal(error_message))?
        }
    }
}
