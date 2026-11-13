//! 图文 / 帖子服务。

use crate::api::Api;
use crate::services::{adapter, to_json};
use serde_json::Value;

pub async fn posts(api: &Api, page: u32, size: u32) -> Result<Value, String> {
    to_json(&adapter(api).posts(page, size).await?)
}

pub async fn post(api: &Api, id: i64) -> Result<Value, String> {
    to_json(&adapter(api).post(id).await?)
}

pub async fn post_playback(api: &Api, id: i64) -> Result<Value, String> {
    to_json(&adapter(api).post_playback(id).await?)
}

pub async fn post_like(api: &Api, id: i64, on: bool) -> Result<Value, String> {
    to_json(&adapter(api).post_like(id, on).await?)
}

pub async fn post_collect(api: &Api, id: i64, on: bool) -> Result<Value, String> {
    to_json(&adapter(api).post_collect(id, on).await?)
}

pub async fn post_comments(api: &Api, id: i64, page: u32, size: u32) -> Result<Value, String> {
    to_json(&adapter(api).post_comments(id, page, size).await?)
}

pub async fn post_categories(api: &Api) -> Result<Value, String> {
    Ok(serde_json::json!({ "items": adapter(api).post_categories().await? }))
}

pub async fn post_category(
    api: &Api,
    route: String,
    page: u32,
    size: u32,
) -> Result<Value, String> {
    to_json(&adapter(api).post_category(&route, page, size).await?)
}
