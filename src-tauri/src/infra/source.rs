//! 门牌（数据源）配置：零校验激活。
//!
//! 用户输入什么就保存什么：只做 trim、缺协议补 `https://`、结尾补 `/`。
//! 不探活、不校验 JSON、不限制域名；真正的请求失败由业务层按普通网络错误返回。

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

pub const ADAPTER_EDGE_V1: &str = "edge_v1";
pub const SOURCE_FILE: &str = "source.json";

/// 落盘的源配置。`base` 是用户输入规整后的地址，`adapter` 固定为 `edge_v1`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConfig {
    pub base: String,
    #[serde(default = "default_adapter")]
    pub adapter: String,
}

fn default_adapter() -> String {
    ADAPTER_EDGE_V1.to_string()
}

impl SourceConfig {
    pub fn new(base: String) -> Self {
        Self {
            base,
            adapter: default_adapter(),
        }
    }
}

/// 给前端的门牌状态；未配置时 `activated=false`，`base` 为空。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateState {
    pub base: String,
    pub adapter: String,
    pub activated: bool,
}

impl Default for GateState {
    fn default() -> Self {
        Self {
            base: String::new(),
            adapter: default_adapter(),
            activated: false,
        }
    }
}

static SOURCE: Lazy<RwLock<Option<SourceConfig>>> = Lazy::new(|| RwLock::new(None));
static SOURCE_FILE_PATH: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

/// 启动时绑定应用数据目录，并读回上次保存的门牌。
pub fn bind(dir: &std::path::Path) {
    let path = dir.join(SOURCE_FILE);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<SourceConfig>(&text) {
            if !config.base.trim().is_empty() {
                if let Ok(mut cur) = SOURCE.write() {
                    *cur = Some(config);
                }
            }
        }
    }
    if let Ok(mut file) = SOURCE_FILE_PATH.write() {
        *file = Some(path);
    }
}

/// 零校验规整：trim → 无协议补 `https://` → 补结尾 `/`。
pub fn normalize_door(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let without_trailing = with_scheme.trim_end_matches('/');
    format!("{without_trailing}/")
}

/// 激活门牌：任何非空输入都放行，写内存并发起落盘。
pub fn gate_open(raw: &str) -> Result<GateState, String> {
    let base = normalize_door(raw);
    if base.is_empty() {
        return Err("请输入接入地址".into());
    }
    let config = SourceConfig::new(base);
    if let Ok(mut cur) = SOURCE.write() {
        *cur = Some(config.clone());
    }
    if let Some(path) = SOURCE_FILE_PATH.read().ok().and_then(|p| p.clone()) {
        let _ = std::fs::write(path, serde_json::to_string(&config).unwrap_or_default());
    }
    crate::playback_log::record(
        "source.changed",
        serde_json::json!({
            "adapter": config.adapter,
            "host": url::Url::parse(&config.base).ok().and_then(|u| u.host_str().map(str::to_owned)),
        }),
    );
    Ok(GateState {
        base: config.base,
        adapter: config.adapter,
        activated: true,
    })
}

/// 当前门牌状态。
pub fn gate_state() -> GateState {
    match SOURCE.read().ok().and_then(|cur| cur.clone()) {
        Some(config) => GateState {
            base: config.base,
            adapter: config.adapter,
            activated: true,
        },
        None => GateState::default(),
    }
}

/// 复位门牌：清空内存并删除落盘配置。
pub fn gate_reset() -> GateState {
    if let Ok(mut cur) = SOURCE.write() {
        *cur = None;
    }
    if let Some(path) = SOURCE_FILE_PATH.read().ok().and_then(|p| p.clone()) {
        let _ = std::fs::remove_file(path);
    }
    GateState::default()
}

/// 当前基址（未配置时为空串）。这是 `net::base()` 的唯一来源。
pub fn base() -> String {
    SOURCE
        .read()
        .ok()
        .and_then(|cur| cur.as_ref().map(|c| c.base.clone()))
        .unwrap_or_default()
}

/// 判断某个请求 origin 是否就是当前门牌（用于凭据只发给当前源）。
pub fn is_current_origin(origin: &str) -> bool {
    let current = base();
    if current.is_empty() {
        return false;
    }
    let a = url::Url::parse(origin)
        .ok()
        .map(|u| u.origin().ascii_serialization());
    let b = url::Url::parse(&current)
        .ok()
        .map(|u| u.origin().ascii_serialization());
    a.is_some() && a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn door_normalization_is_zero_validation() {
        assert_eq!(normalize_door(" example.com "), "https://example.com/");
        assert_eq!(
            normalize_door("https://example.com"),
            "https://example.com/"
        );
        assert_eq!(
            normalize_door("https://example.com/path/?x=1"),
            "https://example.com/path/?x=1/"
        );
        assert_eq!(normalize_door("不会解析的地址"), "https://不会解析的地址/");
        assert_eq!(normalize_door("   "), "");
    }

    #[test]
    fn gate_open_accepts_any_non_empty_address() {
        let state = gate_open(" example.com ").unwrap();
        assert_eq!(state.base, "https://example.com/");
        assert_eq!(state.adapter, ADAPTER_EDGE_V1);
        assert!(state.activated);
        gate_reset();
        assert!(!gate_state().activated);
    }
}
