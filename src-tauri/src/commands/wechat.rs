use tauri::State;
use crate::state::AppState;

// ============ Wechat ============
#[tauri::command]
pub async fn wechat_publish(
    state: State<'_, AppState>,
    account_id: String,
    title: String,
    content: String,
    author: Option<String>,
    cover_media_id: Option<String>,
    publish_type: Option<String>,
) -> Result<String, String> {
    log::info!("[微信发布] 开始发布文章: {} (类型: {:?})", title, publish_type);
    let account = {
        let db = lock_db!(state);
        let accounts = db.get_all_accounts().unwrap_or_default();
        accounts.into_iter().find(|a| a.id == account_id)
            .ok_or("公众号账号不存在")?
    };
    let effective_cover = cover_media_id
        .filter(|id| !id.trim().is_empty())
        .or_else(|| {
            let default_id = account.default_cover_media_id.trim();
            if default_id.is_empty() { None } else { Some(default_id.to_string()) }
        });

    let client = crate::wechat::WechatClient::new(&account.app_id, &account.app_secret);

    // 步骤1：创建草稿
    let mut draft_media_id = None;
    for attempt in 0..2 {
        let use_cached = attempt == 0 && {
            let db = lock_db!(state);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs() as i64;
            db.get_wechat_access_token(&account_id).unwrap_or(None)
                .map(|t| t.expires_at > now).unwrap_or(false)
        };

        let access_token = if use_cached {
            let cached = {
                let db = lock_db!(state);
                db.get_wechat_access_token(&account_id).ok().flatten()
            };
            match cached {
                Some(t) => t.access_token,
                None => {
                    let new_tok = client.fetch_access_token().await
                        .map_err(|e| { log::error!("[微信] 获取token失败: {}", e); e })?;
                    let db = lock_db!(state);
                    db.save_wechat_access_token(&crate::database::WechatAccessToken {
                        account_id: account_id.clone(),
                        access_token: new_tok.access_token.clone(),
                        expires_at: new_tok.expires_at,
                    }).ok();
                    new_tok.access_token
                }
            }
        } else {
            let new_tok = client.fetch_access_token().await
                .map_err(|e| { log::error!("[微信发布] 获取token失败: {}", e); e })?;
            let db = lock_db!(state);
            db.save_wechat_access_token(&crate::database::WechatAccessToken {
                account_id: account_id.clone(),
                access_token: new_tok.access_token.clone(),
                expires_at: new_tok.expires_at,
            }).ok();
            new_tok.access_token
        };

        let result = client.create_draft(&access_token, &title, &content, author.as_deref(), effective_cover.as_deref()).await?;
        if result.errcode == 0 {
            draft_media_id = result.media_id;
            break;
        }
        if result.errcode != 40001 || attempt == 1 {
            log::error!("[微信发布] 创建草稿失败 errcode={} errmsg={}", result.errcode, result.errmsg);
            return Err(format!("创建草稿失败: {} - {}", result.errcode, result.errmsg));
        }
        log::warn!("[微信发布] token失效，刷新重试...");
    }
    let media_id = draft_media_id.ok_or("创建草稿失败：未获取到 media_id".to_string())?;
    log::info!("[微信发布] 草稿创建成功 media_id={}", media_id);

    log::info!("[微信发布] 草稿创建成功 media_id={}", media_id);

    // 步骤2：如果是群发，提交发布
    if publish_type.as_deref() == Some("mass") {
        for attempt in 0..2 {
            let access_token = if attempt == 0 {
                let db = lock_db!(state);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs() as i64;
                db.get_wechat_access_token(&account_id).unwrap_or(None)
                    .filter(|t| t.expires_at > now)
                    .map(|t| t.access_token)
            } else {
                let new_tok = client.fetch_access_token().await
                    .map_err(|e| { log::error!("[微信发布] 刷新token失败: {}", e); e })?;
                let db = lock_db!(state);
                db.save_wechat_access_token(&crate::database::WechatAccessToken {
                    account_id: account_id.clone(),
                    access_token: new_tok.access_token.clone(),
                    expires_at: new_tok.expires_at,
                }).ok();
                Some(new_tok.access_token)
            };

            let access_token = access_token.ok_or("无法获取有效的 access_token".to_string())?;
            let result = client.publish_draft(&access_token, &media_id).await?;
            if result.errcode == 0 {
                log::info!("[微信发布] 群发成功 media_id={}", media_id);
                return Ok("发布成功".to_string());
            }
            if result.errcode != 40001 || attempt == 1 {
                log::error!("[微信发布] 群发失败 errcode={} errmsg={}", result.errcode, result.errmsg);
                return Err(format!("群发失败: {} - {}", result.errcode, result.errmsg));
            }
            log::warn!("[微信发布] token失效，刷新重试...");
        }
        return Err("群发失败".into());
    }

    Ok("草稿已保存".to_string())
}

// ============ Wechat Media Upload ============
#[tauri::command]
pub async fn wechat_upload_media(
    state: State<'_, AppState>,
    account_id: String,
    name: String,
    media_type: String,
    data: Vec<u8>,
) -> Result<serde_json::Value, String> {
    let (app_id, app_secret) = {
        let db = lock_db!(state);
        let accounts = db.get_all_accounts().unwrap_or_default();
        let account = accounts.iter().find(|a| a.id == account_id)
            .ok_or("公众号账号不存在")?;
        (account.app_id.clone(), account.app_secret.clone())
    };

    let client = crate::wechat::WechatClient::new(&app_id, &app_secret);

    // 获取 token 并上传，遇到 40001 自动刷新重试一次
    for attempt in 0..2 {
        let use_cached = attempt == 0 && {
            let db = lock_db!(state);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs() as i64;
            db.get_wechat_access_token(&account_id).unwrap_or(None)
                .map(|t| t.expires_at > now).unwrap_or(false)
        };

        let access_token = if use_cached {
            let cached = {
                let db = lock_db!(state);
                db.get_wechat_access_token(&account_id).ok().flatten()
            };
            match cached {
                Some(t) => t.access_token,
                None => {
                    let new_tok = client.fetch_access_token().await
                        .map_err(|e| { log::error!("[微信] 获取token失败: {}", e); e })?;
                    let db = lock_db!(state);
                    db.save_wechat_access_token(&crate::database::WechatAccessToken {
                        account_id: account_id.clone(),
                        access_token: new_tok.access_token.clone(),
                        expires_at: new_tok.expires_at,
                    }).ok();
                    new_tok.access_token
                }
            }
        } else {
            let new_tok = client.fetch_access_token().await
                .map_err(|e| { log::error!("[微信上传] 获取token失败: {}", e); e })?;
            let db = lock_db!(state);
            db.save_wechat_access_token(&crate::database::WechatAccessToken {
                account_id: account_id.clone(),
                access_token: new_tok.access_token.clone(),
                expires_at: new_tok.expires_at,
            }).ok();
            new_tok.access_token
        };

        let result = client.upload_media(&access_token, &media_type, &data, &name).await?;
        if result.errcode.is_none() || result.errcode == Some(0) {
            log::info!("[微信上传] 上传成功 name=\"{}\"", name);
            return Ok(serde_json::json!({ "mediaId": result.media_id }));
        }
        if result.errcode != Some(40001) || attempt == 1 {
            log::error!("[微信上传] 上传失败 name=\"{}\" response={:?}", name, result);
            return Err(format!("上传失败: {:?} - {:?}", result.errcode, result.errmsg));
        }
        log::warn!("[微信上传] token失效，刷新重试...");
    }

    Err("上传失败".into())
}

// 配置公众号时尚未保存账号，需直接使用表单中的 AppID/AppSecret 上传默认封面
#[tauri::command]
pub async fn wechat_upload_media_with_app(
    app_id: String,
    app_secret: String,
    name: String,
    media_type: String,
    data: Vec<u8>,
) -> Result<serde_json::Value, String> {
    if app_id.trim().is_empty() || app_secret.trim().is_empty() {
        return Err("AppID 和 AppSecret 不能为空".to_string());
    }

    let client = crate::wechat::WechatClient::new(&app_id, &app_secret);

    for attempt in 0..2 {
        let token = client.fetch_access_token().await
            .map_err(|e| { log::error!("[微信上传] 获取token失败: {}", e); e })?;
        let access_token = &token.access_token;
        let result = client.upload_media(access_token, &media_type, &data, &name).await?;
        if result.errcode.is_none() || result.errcode == Some(0) {
            log::info!("[微信上传] 上传成功 name=\"{}\"", name);
            return Ok(serde_json::json!({ "mediaId": result.media_id }));
        }
        if result.errcode != Some(40001) || attempt == 1 {
            log::error!("[微信上传] 上传失败 name=\"{}\" response={:?}", name, result);
            return Err(format!("上传失败: {:?} - {:?}", result.errcode, result.errmsg));
        }
        log::warn!("[微信上传] token失效，刷新重试...");
    }

    Err("上传失败".into())
}
