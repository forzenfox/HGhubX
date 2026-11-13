//! 目录 / 搜索 / 榜单服务。

use crate::api::Api;
use crate::services::{adapter, to_json};
use serde_json::{json, Value};

pub async fn feed(api: &Api, page: u32, size: u32) -> Result<Value, String> {
    to_json(&adapter(api).feed(page, size).await?)
}

pub async fn topics(api: &Api) -> Result<Value, String> {
    Ok(json!({ "items": adapter(api).topics().await? }))
}

pub async fn topic(api: &Api, slug: String) -> Result<Value, String> {
    to_json(&adapter(api).topic(&slug).await?)
}

pub async fn tags(api: &Api) -> Result<Value, String> {
    Ok(json!({ "categories": adapter(api).tags().await? }))
}

pub async fn tag_videos(api: &Api, tag: String, page: u32, size: u32) -> Result<Value, String> {
    to_json(&adapter(api).tag_videos(&tag, page, size).await?)
}

pub async fn category(api: &Api, cate: String, page: u32, size: u32) -> Result<Value, String> {
    to_json(&adapter(api).category(&cate, page, size).await?)
}

pub async fn rank(api: &Api, kind: String) -> Result<Value, String> {
    let kind = match kind.as_str() {
        "recommend" | "potential" => kind.as_str(),
        _ => "hot",
    };
    Ok(json!({ "type": kind, "items": adapter(api).rank(kind).await? }))
}

pub async fn search(
    api: &Api,
    q: String,
    page: u32,
    size: u32,
    kind: String,
) -> Result<Value, String> {
    adapter(api).search(&q, page, size, &kind).await
}

pub async fn search_hot(api: &Api) -> Result<Value, String> {
    Ok(json!({ "items": adapter(api).search_hot().await? }))
}
