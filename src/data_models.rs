#![allow(dead_code)]

use mongodb::bson::{oid::ObjectId, serde_helpers::serialize_u64_as_i64, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub personal_boards: Vec<Board>,
    pub group_boards: Vec<ObjectId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Board {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub category: BoardKind,
    pub author_username: String,
    pub author_id: ObjectId,
    pub pins: Vec<ObjectId>,
    #[serde(serialize_with = "serialize_u64_as_i64")]
    pub views: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pin {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub author_username: String,
    pub author_id: ObjectId,
    pub date_created: DateTime,
    pub url: String,
    pub image_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BoardKind {
    Personal,
    Group,
}
