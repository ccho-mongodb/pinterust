use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, to_document},
    Client, Collection,
};
use serde::Deserialize;
use tera::{Context, Tera, Value};

use super::data_models::{Board, BoardKind, Pin, User};
use super::user::set_view;

#[derive(Deserialize)]
pub struct BoardForm {
    title: String,
}

impl BoardForm {
    pub fn into_board(&self, session: &Session, kind: BoardKind) -> Board {
        Board {
            id: ObjectId::new(),
            title: self.title.clone(),
            category: kind,
            author_username: session.get("username").unwrap().unwrap(),
            author_id: session.get("user_id").unwrap().unwrap(),
            pins: Vec::new(),
            views: 0,
        }
    }
}

#[post("/new_personal_board")]
async fn new_personal_board(
    board: web::Form<BoardForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let board = board.into_board(&session, BoardKind::Personal);
    let users: Collection<User> = client.database("pinterust").collection_with_type("users");
    let update = doc! { "$addToSet": { "personal_boards": to_document(&board).unwrap() } };
    let author: String = session.get("username").unwrap().unwrap();
    users
        .update_one(doc! { "username": author }, update, None)
        .await
        .unwrap();

    set_view(client, tera, session).await
}

#[post("/new_group_board")]
async fn new_group_board(
    board: web::Form<BoardForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let board = board.into_board(&session, BoardKind::Group);

    let boards: Collection<Board> = client.database("pinterust").collection_with_type("boards");
    boards.insert_one(board.clone(), None).await.unwrap();

    let users: Collection<User> = client.database("pinterust").collection_with_type("users");
    let update = doc! { "$addToSet": { "group_boards": board.id.clone() } };
    let author: String = session.get("username").unwrap().unwrap();
    users
        .update_one(doc! { "username": author }, update, None)
        .await
        .unwrap();

    set_view(client, tera, session).await
}

#[post("get_personal_board")]
pub async fn get_personal_board(
    form: web::Form<BoardForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let username: String = session.get("username").unwrap().unwrap();
    let users: Collection<User> = client.database("pinterust").collection_with_type("users");
    let user = users
        .find_one(doc! { "username": username }, None)
        .await
        .unwrap()
        .unwrap();

    // TODO query this board directly from the users collection
    let board = user
        .personal_boards
        .iter()
        .filter(|board| board.title == form.title)
        .next()
        .unwrap();
    session.set("board", board).unwrap();

    view_board(client, tera, session).await
}

#[post("get_group_board")]
pub async fn get_group_board(
    form: web::Form<BoardForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let boards: Collection<Board> = client.database("pinterust").collection_with_type("boards");
    let board = boards
        .find_one(doc! { "title": &form.title }, None)
        .await
        .unwrap()
        .unwrap();
    session.set("board", board).unwrap();
    // TODO increment view count

    view_board(client, tera, session).await
}

pub async fn view_board(
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let board: Board = session.get("board").unwrap().unwrap();

    let mut context = Context::new();

    context.insert("board", &board);

    let pins: Collection<Pin> = client.database("pinterust").collection_with_type("pins");
    let filter = doc! {
        "id": { "$in": board.pins }
    };
    let pins: Vec<Pin> = pins
        .find(filter, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    context.insert("pins", &pins);

    let body = tera.render("board.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}
