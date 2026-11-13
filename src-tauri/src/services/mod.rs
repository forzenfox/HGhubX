//! 业务服务：只编排领域命令，所有上游细节交给适配器。

mod account_service;
mod catalog_service;
mod playback_service;
mod post_service;

use crate::adapters::edge_v1::EdgeV1Adapter;
use crate::api::Api;
use crate::domain::DomainCommand;
use crate::infra::source;
use serde_json::Value;

pub(crate) fn adapter(api: &Api) -> EdgeV1Adapter {
    let origin = source::base();
    let token = api.session.token(&origin);
    EdgeV1Adapter::new(origin, token)
}

pub(crate) fn to_json<T: serde::Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

/// `data_call` 的总入口：领域命令 → service → EdgeV1Adapter。
pub async fn dispatch(api: &Api, command: DomainCommand) -> Result<Value, String> {
    match command {
        DomainCommand::Boot => Ok(serde_json::json!({
            "base": source::base(),
            "gate": source::gate_state(),
            "user": api.session.user(&source::base()),
            "logged_in": api.session.token(&source::base()).is_some(),
            "strict_security": crate::network_policy::strict_security(),
            "cli_relaxed": false,
        })),
        DomainCommand::Login { username, password } => {
            account_service::login(api, username, password).await
        }
        DomainCommand::Register {
            username,
            password,
            confirm_password,
            email,
        } => account_service::register(api, username, password, confirm_password, email).await,
        DomainCommand::Logout => account_service::logout(api).await,
        DomainCommand::Me => account_service::me(api).await,
        DomainCommand::Feed { page, size } => catalog_service::feed(api, page, size).await,
        DomainCommand::Topics => catalog_service::topics(api).await,
        DomainCommand::Topic { slug } => catalog_service::topic(api, slug).await,
        DomainCommand::Tags => catalog_service::tags(api).await,
        DomainCommand::TagVideos { tag, page, size } => {
            catalog_service::tag_videos(api, tag, page, size).await
        }
        DomainCommand::Category { cate, page, size } => {
            catalog_service::category(api, cate, page, size).await
        }
        DomainCommand::Rank { kind } => catalog_service::rank(api, kind).await,
        DomainCommand::Search {
            q,
            page,
            size,
            kind,
        } => catalog_service::search(api, q, page, size, kind).await,
        DomainCommand::SearchHot => catalog_service::search_hot(api).await,
        DomainCommand::Posts { page, size } => post_service::posts(api, page, size).await,
        DomainCommand::Post { id } => post_service::post(api, id).await,
        DomainCommand::PostPlayback { id } => post_service::post_playback(api, id).await,
        DomainCommand::PostLike { id, on } => post_service::post_like(api, id, on).await,
        DomainCommand::PostCollect { id, on } => post_service::post_collect(api, id, on).await,
        DomainCommand::PostComments { id, page, size } => {
            post_service::post_comments(api, id, page, size).await
        }
        DomainCommand::PostCategories => post_service::post_categories(api).await,
        DomainCommand::PostCategory { route, page, size } => {
            post_service::post_category(api, route, page, size).await
        }
        DomainCommand::Video { id } => playback_service::video(api, id).await,
        DomainCommand::Play { id, ep } => playback_service::play(api, id, ep).await,
        DomainCommand::Comments { id, ep, page, size } => {
            playback_service::comments(api, id, ep, page, size).await
        }
        DomainCommand::CommentSend {
            id,
            ep,
            text,
            reply_id,
        } => playback_service::comment_send(api, id, ep, text, reply_id).await,
        DomainCommand::Like { id, ep, on } => playback_service::like(api, id, ep, on).await,
        DomainCommand::Collect { id, on } => playback_service::collect(api, id, on).await,
        DomainCommand::Follow { uid, on } => playback_service::follow(api, uid, on).await,
        DomainCommand::Related { id } => playback_service::related(api, id).await,
        DomainCommand::Following { page, size } => {
            account_service::following(api, page, size).await
        }
        DomainCommand::User { id } => account_service::user(api, id).await,
        DomainCommand::UserStats { id } => account_service::user_stats(api, id).await,
        DomainCommand::UserVideos { id, page, size } => {
            account_service::user_videos(api, id, page, size).await
        }
    }
}
