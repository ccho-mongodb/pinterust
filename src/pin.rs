use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Collection,
};
use serde::Deserialize;
use tera::Tera;

use crate::board::view_board;

use super::data_models::{Board, Pin, User};

#[derive(Deserialize)]
struct PinForm {
    title: String,
    url: String,
    image_url: String,
}

impl PinForm {
    pub fn into_pin(&self, session: &Session) -> Pin {
        Pin {
            id: ObjectId::new(),
            title: self.title.clone(),
            author_username: session.get("username").unwrap().unwrap(),
            author_id: session.get("user_id").unwrap().unwrap(),
            date_created: Utc::now().into(),
            url: self.url.clone(),
            image_url: self.image_url.clone(),
        }
    }
}

#[post("/add_pin")]
async fn add_pin(
    pin: web::Form<PinForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let pins: Collection<Pin> = client.database("pinterust").collection_with_type("pins");
    let pin: Pin = pin.into_pin(&session);
    pins.insert_one(pin.clone(), None).await.unwrap();

    // TODO add pin to current board's pins list

    view_board(client, tera, session).await
}
