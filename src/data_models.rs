#![allow(dead_code)]

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

pub(crate) struct User {
    id: ObjectId,
    username: String,
    personal_boards: Vec<Board>,
    group_boards: Vec<ObjectId>,
}

pub(crate) struct Board {
    id: ObjectId,
    title: String,
    category: BoardKind,
    author_username: String,
    author_id: ObjectId,
    pins: Vec<ObjectId>,
    views: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Pin {
    pub id: ObjectId,
    pub title: String,
    pub author_username: String,
    pub author_id: ObjectId,
    pub date_created: DateTime,
    pub url: String,
    pub image_url: String,
}

pub(crate) enum BoardKind {
    Personal,
    Group,
}
