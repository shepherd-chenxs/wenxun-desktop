//! 热点雷达：聚合各平台公开热榜，无需登录凭证。
//! 数据源实现参考 DailyHotApi（imsyy/DailyHotApi），全部为公开接口直连。

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
const CACHE_TTL: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Serialize)]
pub struct HotItem {
    pub rank: u32,
    pub title: String,
    pub heat: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotSource {
    pub key: String,
    pub name: String,
}

pub const SOURCE_KEYS: &[(&str, &str)] = &[
    ("weibo", "微博"),
    ("zhihu", "知乎"),
    ("douyin", "抖音"),
    ("baidu", "百度"),
    ("toutiao", "头条"),
    ("bilibili", "B站"),
];

static CACHE: LazyLock<Mutex<HashMap<String, (Instant, Vec<HotItem>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn trim_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

async fn get_json(
    client: &reqwest::Client,
    url: &str,
    referer: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut rb = client.get(url);
    if let Some(r) = referer {
        rb = rb.header("Referer", r);
    }
    let resp = rb
        .send()
        .await
        .map_err(|e| format!("请求失败: {}（请检查网络后重试）", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("接口返回解析失败: {}", e))
}

/// 递归收集 JSON 中所有带 "word" 字段的对象（百度热榜结构嵌套不稳定，用此兜底）
fn collect_objects(value: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    match value {
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_objects(v, out);
            }
        }
        serde_json::Value::Object(map) => {
            if map.contains_key("word") {
                out.push(value.clone());
            }
            for v in map.values() {
                collect_objects(v, out);
            }
        }
        _ => {}
    }
}

async fn weibo(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(client, "https://weibo.com/ajax/side/hotSearch", Some("https://weibo.com/")).await?;
    let realtime = v["data"]["realtime"].as_array().cloned().unwrap_or_default();
    let mut items = Vec::new();
    for it in realtime {
        let word = it["word"].as_str().unwrap_or("").trim();
        if word.is_empty() {
            continue;
        }
        let heat = it["num"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let url = format!("https://s.weibo.com/weibo?q=%23{}%23", urlencoding::encode(word));
        items.push((word.to_string(), heat, url));
    }
    Ok(items)
}

async fn zhihu(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(client, "https://api.zhihu.com/topstory/hot-list?limit=50", None).await?;
    let list = v["data"].as_array().cloned().unwrap_or_default();
    let mut items = Vec::new();
    for it in list {
        let title = it["target"]["title"].as_str().unwrap_or("");
        if title.is_empty() {
            continue;
        }
        // api.zhihu.com/questions/123 → www.zhihu.com/question/123
        let raw = it["target"]["url"].as_str().unwrap_or("");
        let url = raw
            .split('/')
            .last()
            .map(|id| format!("https://www.zhihu.com/question/{}", id))
            .unwrap_or_else(|| "https://www.zhihu.com/hot".into());
        items.push((trim_text(title), it["detail_text"].as_str().unwrap_or("").to_string(), url));
    }
    Ok(items)
}

async fn douyin(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(
        client,
        "https://www.douyin.com/aweme/v1/web/hot/search/list/?source=6&pc_client_type=1",
        Some("https://www.douyin.com/"),
    ).await?;
    let list = v["data"]["word_list"].as_array().cloned().unwrap_or_default();
    let mut items = Vec::new();
    for it in list {
        let word = it["word"].as_str().unwrap_or("").trim();
        if word.is_empty() {
            continue;
        }
        let heat = it["hot_value"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let url = it["event_url"].as_str().unwrap_or("").to_string();
        items.push((word.to_string(), heat, url));
    }
    Ok(items)
}

async fn baidu(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(client, "https://top.baidu.com/api/board?platform=wise&tab=realtime", None).await?;
    let mut objs = Vec::new();
    collect_objects(&v["data"]["cards"], &mut objs);
    let mut items = Vec::new();
    for it in objs {
        let word = it["word"].as_str().unwrap_or("").trim();
        if word.is_empty() {
            continue;
        }
        let heat = it["hot_score"]
            .as_u64()
            .map(|n| n.to_string())
            .unwrap_or_default();
        let url = it["url"].as_str().unwrap_or("").to_string();
        items.push((trim_text(word), heat, url));
    }
    Ok(items)
}

async fn toutiao(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(client, "https://www.toutiao.com/hot-event/hot-board/?origin=toutiao_pc", None).await?;
    let list = v["data"].as_array().cloned().unwrap_or_default();
    let mut items = Vec::new();
    for it in list {
        let title = it["Title"].as_str().unwrap_or("").trim();
        if title.is_empty() {
            continue;
        }
        let heat = it["HotValue"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let url = it["Url"].as_str().unwrap_or("").to_string();
        items.push((trim_text(title), heat, url));
    }
    Ok(items)
}

async fn bilibili(client: &reqwest::Client) -> Result<Vec<(String, String, String)>, String> {
    let v = get_json(
        client,
        "https://api.bilibili.com/x/web-interface/search/square?limit=50",
        Some("https://www.bilibili.com/"),
    ).await?;
    let list = v["data"]["trending"]["list"].as_array().cloned().unwrap_or_default();
    let mut items = Vec::new();
    for it in list {
        let word = it["show_name"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "#")
            .or_else(|| it["keyword"].as_str())
            .unwrap_or("")
            .trim()
            .trim_matches('#')
            .to_string();
        if word.is_empty() {
            continue;
        }
        let heat = it["hot_score"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let url = format!("https://search.bilibili.com/all?keyword={}", urlencoding::encode(&word));
        items.push((word, heat, url));
    }
    Ok(items)
}

/// 供定时任务等内部模块调用的取榜接口，返回 (标题, 热度, 链接) 列表
pub async fn fetch_hot_items(source: &str) -> Result<Vec<(String, String, String)>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(UA)
        .no_proxy()
        .build()
        .map_err(|e| format!("创建请求失败: {}", e))?;
    match source {
        "weibo" => weibo(&client).await,
        "zhihu" => zhihu(&client).await,
        "douyin" => douyin(&client).await,
        "baidu" => baidu(&client).await,
        "toutiao" => toutiao(&client).await,
        "bilibili" => bilibili(&client).await,
        _ => Err(format!("未知的热榜源: {}", source)),
    }
}

/// 供定时任务使用：从指定热榜源实时取榜，随机返回榜单前 20 名中的话题标题。
pub async fn pick_hot_topic(source: &str) -> Result<String, String> {
    let source_name = SOURCE_KEYS
        .iter()
        .find(|(key, _)| *key == source)
        .map(|(_, name)| *name)
        .unwrap_or(source);
    let items = fetch_hot_items(source).await?;
    if items.is_empty() {
        return Err(format!("{}热榜当前没有可选话题", source_name));
    }

    let mut candidates: Vec<String> = items
        .into_iter()
        .map(|(title, _, _)| title)
        .collect();

    candidates.truncate(20);

    candidates
        .choose(&mut rand::thread_rng())
        .cloned()
        .ok_or_else(|| format!("{}热榜暂无可选话题", source_name))
}

#[tauri::command]
pub async fn get_hot_list(source: String) -> Result<Vec<HotItem>, String> {
    if !SOURCE_KEYS.iter().any(|(k, _)| *k == source) {
        return Err(format!("未知的热榜源: {}", source));
    }

    if let Ok(cache) = CACHE.lock() {
        if let Some((at, items)) = cache.get(&source) {
            if at.elapsed() < CACHE_TTL {
                return Ok(items.clone());
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(UA)
        .no_proxy()
        .build()
        .map_err(|e| format!("创建请求失败: {}", e))?;

    let raw = match source.as_str() {
        "weibo" => weibo(&client).await?,
        "zhihu" => zhihu(&client).await?,
        "douyin" => douyin(&client).await?,
        "baidu" => baidu(&client).await?,
        "toutiao" => toutiao(&client).await?,
        "bilibili" => bilibili(&client).await?,
        _ => return Err(format!("未知的热榜源: {}", source)),
    };

    if raw.is_empty() {
        return Err("热榜返回为空，该源可能暂时不可用".to_string());
    }

    let items: Vec<HotItem> = raw
        .into_iter()
        .enumerate()
        .map(|(i, (title, heat, url))| HotItem {
            rank: (i + 1) as u32,
            title,
            heat,
            url,
        })
        .collect();

    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(source, (Instant::now(), items.clone()));
    }
    Ok(items)
}

#[tauri::command]
pub fn get_hot_sources() -> Vec<HotSource> {
    SOURCE_KEYS
        .iter()
        .map(|(key, name)| HotSource { key: key.to_string(), name: name.to_string() })
        .collect()
}
