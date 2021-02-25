use actix_session::Session;
use actix_web::{get, post, web, HttpResponse};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Collection,
};
use serde::Deserialize;
use tera::{Context, Tera};

use super::data_models::{Board, User};

#[get("/")]
pub async fn login(tera: web::Data<Tera>) -> HttpResponse {
    let body = tera.render("login.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[derive(Deserialize)]
struct UserForm {
    username: String,
}

#[post("/set_user")]
async fn set_user(
    user: web::Form<UserForm>,
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    session.set("username", &user.username).unwrap();

    let users: Collection<User> = client.database("pinterust").collection_with_type("users");
    let user = match users
        .find_one(doc! { "username": &user.username }, None)
        .await
        .unwrap()
    {
        Some(user) => user,
        None => {
            let user = User {
                id: ObjectId::new(),
                username: user.username.clone(),
                personal_boards: Vec::new(),
                group_boards: Vec::new(),
            };
            users.insert_one(user.clone(), None).await.unwrap();
            user
        }
    };

    session.set("user_id", user.id).unwrap();

    set_view(client, tera, session).await
}

pub async fn set_view(
    client: web::Data<Client>,
    tera: web::Data<Tera>,
    session: Session,
) -> HttpResponse {
    let users: Collection<User> = client.database("pinterust").collection_with_type("users");
    let username: String = session.get("username").unwrap().unwrap();
    let user = users
        .find_one(doc! { "username": username }, None)
        .await
        .unwrap()
        .unwrap();

    let mut context = Context::new();

    context.insert("personal_boards", &user.personal_boards);

    let boards: Collection<Board> = client.database("pinterust").collection_with_type("boards");
    let query = doc! { "id": { "$in": user.group_boards } };
    let group_boards: Vec<Board> = boards
        .find(query, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    context.insert("group_boards", &group_boards);

    let body = tera.render("user.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}
