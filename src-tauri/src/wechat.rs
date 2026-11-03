use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatAccessToken {
    pub access_token: Option<String>,
    pub expires_in: Option<i32>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatPublishResponse {
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
    pub media_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatSubmitResponse {
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
    pub publish_id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatUploadMediaResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub media_id: Option<String>,
    pub created_at: Option<i64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WechatAccessTokenData {
    pub access_token: String,
    pub expires_at: i64,
}

pub struct WechatClient {
    app_id: String,
    app_secret: String,
}

impl WechatClient {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        WechatClient {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }

    pub async fn fetch_access_token(&self) -> Result<WechatAccessTokenData, String> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.app_id, self.app_secret
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| e.to_string())?;
        let response = client.get(&url).send()
            .await
            .map_err(|e| e.to_string())?;

        let body_text = response.text().await.map_err(|e| e.to_string())?;
        log::debug!("[微信] fetch_access_token 原始响应: {}", body_text);

        let token: WechatAccessToken = serde_json::from_str(&body_text)
            .map_err(|e| format!("解析access_token响应失败: {} - body: {}", e, body_text))?;

        if let Some(errcode) = token.errcode {
            if errcode != 0 {
                return Err(format!(
                    "获取access_token失败: errcode={} errmsg={:?} 原始响应: {}",
                    errcode, token.errmsg, body_text
                ));
            }
        }

        let access_token = token.access_token.clone().unwrap_or_default();
        let expires_in = token.expires_in.unwrap_or(7200);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let expires_at = now + expires_in as i64 - 300;

        Ok(WechatAccessTokenData {
            access_token,
            expires_at,
        })
    }

    pub async fn create_draft(
        &self,
        access_token: &str,
        title: &str,
        content: &str,
        author: Option<&str>,
        cover_media_id: Option<&str>,
    ) -> Result<WechatPublishResponse, String> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/draft/add?access_token={}",
            access_token
        );

        let articles = serde_json::json!([{
            "title": title,
            "author": author.unwrap_or(""),
            "digest": "",
            "content": content,
            "content_source_url": "",
            "thumb_media_id": cover_media_id.unwrap_or(""),
            "need_open_comment": 0,
            "only_fans_can_comment": 0
        }]);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| e.to_string())?;
        let response = client.post(&url)
            .json(&serde_json::json!({ "articles": articles }))
            .send().await.map_err(|e| e.to_string())?;

        let body_text = response.text().await.map_err(|e| e.to_string())?;
        log::debug!("[微信] create_draft 原始响应: {}", body_text);

        let result: WechatPublishResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("解析草稿响应失败: {} - body: {}", e, body_text))?;

        Ok(result)
    }

    pub async fn publish_draft(
        &self,
        access_token: &str,
        media_id: &str,
    ) -> Result<WechatSubmitResponse, String> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/freepublish/submit?access_token={}",
            access_token
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| e.to_string())?;
        let response = client.post(&url)
            .json(&serde_json::json!({ "media_id": media_id }))
            .send().await.map_err(|e| e.to_string())?;

        let body_text = response.text().await.map_err(|e| e.to_string())?;
        log::debug!("[微信] publish_draft 原始响应: {}", body_text);

        let result: WechatSubmitResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("解析发布响应失败: {} - body: {}", e, body_text))?;

        Ok(result)
    }

    pub async fn upload_media(
        &self,
        access_token: &str,
        media_type: &str,
        media_data: &[u8],
        file_name: &str,
    ) -> Result<WechatUploadMediaResponse, String> {
        // 使用永久素材接口，media_id 可用于 add_news
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/material/add_material?access_token={}&type={}",
            access_token, media_type
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| e.to_string())?;
        let response = client
            .post(&url)
            .multipart(
                reqwest::multipart::Form::new()
                    .part(
                        "media",
                        reqwest::multipart::Part::bytes(media_data.to_vec())
                            .file_name(file_name.to_string())
                    )
            )
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body_text = response.text().await.map_err(|e| e.to_string())?;
        log::debug!("[微信] upload_media 原始响应: {}", body_text);

        let result: WechatUploadMediaResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("解析上传响应失败: {} - body: {}", e, body_text))?;

        Ok(result)
    }
}