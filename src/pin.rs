use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Result,
    results::InsertOneResult,
    Client, Collection,
};
use serde::Deserialize;
use tera::Tera;

use crate::board::view_board;

use super::{
    data_models::{Board, Pin, User},
    DB_NAME,
};

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
    let pin: Pin = pin.into_pin(&session);
    insert_pin(&client, pin.clone()).await.unwrap();

    let board: Board = session.get("board").unwrap().unwrap();
    let boards: Collection<Board> = client.database("pinterust").collection_with_type("boards");
    let filter = doc! { "_id": board.id };
    let update = doc! { "$push": { "pins": pin.id } };
    boards.update_one(filter, update, None).await.unwrap();

    view_board(client, tera, session).await
}

// TODO #1
async fn insert_pin(client: &Client, pin: Pin) -> Result<InsertOneResult> {
    let db = client.database(DB_NAME);
    let coll: Collection<Pin> = db.collection_with_type("pins");
    coll.insert_one(pin, None).await
}
