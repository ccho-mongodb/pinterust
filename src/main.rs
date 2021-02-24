#![allow(dead_code, unused_imports)]

use std::collections::HashMap;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime, Document},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Result, Tera, Value};

mod data_models;

use data_models::Pin;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("INSERT-URI-HERE").await.unwrap();

    let tera = Tera::new("html/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .data(client.clone())
            .data(tera.clone())
            .service(index)
            .service(form)
    })
    .bind("127.0.0.1:8083")?
    .run()
    .await
}

#[get("/")]
async fn index(tera: web::Data<Tera>, client: web::Data<Client>) -> HttpResponse {
    let pins: Collection<Pin> = client.database("pinterust").collection_with_type("pins");
    // TODO load current pins into page
    let _pins = match pins.find(None, None).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::Ok().body("error occurred when loading pins"),
    };
    let body = tera.render("simple_index.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[derive(Deserialize)]
pub struct PinForm {
    title: String,
    url: String,
    image_url: String,
}

impl From<PinForm> for Pin {
    fn from(pin_form: PinForm) -> Self {
        Pin {
            id: ObjectId::new(),
            title: pin_form.title,
            // TODO use current user's info
            author_username: "".to_string(),
            author_id: ObjectId::new(),
            date_created: Utc::now().into(),
            url: pin_form.url,
            image_url: pin_form.image_url,
        }
    }
}

#[post("/form")]
async fn form(pin: web::Form<PinForm>, client: web::Data<Client>) -> HttpResponse {
    let pins: Collection<Pin> = client.database("pinterust").collection_with_type("pins");
    let pin: Pin = pin.into_inner().into();
    match pins.insert_one(pin, None).await {
        Ok(_) => HttpResponse::Ok().body("pin added!"),
        Err(_) => HttpResponse::Ok().body("insert failed"),
    }
}
