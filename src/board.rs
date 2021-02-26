use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, to_document},
    error::Result,
    results::UpdateResult,
    Client, Collection,
};
use serde::Deserialize;
use tera::{Context, Tera, Value};

use super::data_models::{Board, BoardKind, Pin, User};
use super::user::set_view;
use super::DB_NAME;

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

    view_board(client, tera, session).await
}

pub async fn view_board(
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let mut context = Context::new();

    let board: Board = session.get("board").unwrap().unwrap();
    context.insert("board", &board);

    let pins = match board.category {
        BoardKind::Personal => get_group_board_pins(&client, board.clone()).await.unwrap(),
        BoardKind::Group => {
            let user_id: ObjectId = session.get("user_id").unwrap().unwrap();
            let pins = get_personal_board_pins(&client, &board.id, &user_id)
                .await
                .unwrap();
            dbg!("{}", &pins);
            pins
        }
    };
    context.insert("pins", &pins);

    match board.category {
        BoardKind::Personal => {
            // TODO increment view for personal board
        }
        BoardKind::Group => {
            increment_group_board_count(&client, &board.id)
                .await
                .unwrap();
        }
    }

    let body = tera.render("board.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

async fn get_personal_board_pins(
    client: &Client,
    board_id: &ObjectId,
    user_id: &ObjectId,
) -> Result<Vec<Pin>> {
    let coll: Collection<User> = client.database("DB_NAME").collection_with_type("users");
    let filter = doc! {
        "_id": user_id
    };
    let user = coll.find_one(filter, None).await?.unwrap();
    let pins = user
        .personal_boards
        .iter()
        .cloned()
        .filter(|board| &board.id == board_id)
        .next()
        .unwrap()
        .pins;

    let coll: Collection<Pin> = client.database(DB_NAME).collection_with_type("pins");
    let filter = doc! {
        "_id": { "$in": pins }
    };
    let cursor = coll.find(filter, None).await?;
    cursor.try_collect().await
}

// TODO #2
async fn get_group_board_pins(client: &Client, board: Board) -> Result<Vec<Pin>> {
    let coll: Collection<Pin> = client.database(DB_NAME).collection_with_type("pins");
    let filter = doc! {
        "_id": { "$in": board.pins }
    };
    let cursor = coll.find(filter, None).await?;
    cursor.try_collect().await
}

// TODO #3
async fn increment_group_board_count(client: &Client, board_id: &ObjectId) -> Result<UpdateResult> {
    let boards: Collection<Board> = client.database(DB_NAME).collection_with_type("boards");
    let filter = doc! {
        "_id": board_id
    };
    let update = doc! {
        "$inc": { "views": 1 }
    };
    boards.update_one(filter, update, None).await
}
