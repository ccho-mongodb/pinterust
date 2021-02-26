#![allow(dead_code, unused_imports)]

use std::{collections::HashMap, env};

use actix_session::{CookieSession, Session};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId, to_document, DateTime, Document},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Result, Tera, Value};

mod board;
mod data_models;
mod pin;
mod user;

use board::{get_group_board, get_personal_board, new_group_board, new_personal_board, view_board};
use data_models::{Board, BoardKind, Pin, User};
use pin::add_pin;
use user::{login, set_user};

pub static DB_NAME: &str = "pinterust";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = env!("MONGODB_URI");
    let client = Client::with_uri_str(uri).await.unwrap();
    let tera = Tera::new("html/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .data(client.clone())
            .data(tera.clone())
            .wrap(CookieSession::signed(&[0; 32]))
            .service(load_data)
            .service(set_user)
            .service(login)
            .service(new_personal_board)
            .service(new_group_board)
            .service(get_personal_board)
            .service(get_group_board)
            .service(add_pin)
    })
    .bind("127.0.0.1:8083")?
    .run()
    .await
}

#[get("/")]
async fn load_data() -> HttpResponse {
    
}
