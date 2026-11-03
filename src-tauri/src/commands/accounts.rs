use tauri::State;
use crate::database::WechatAccount;
use crate::state::AppState;

// ============ Wechat Accounts ============
#[tauri::command]
pub fn get_accounts(state: State<AppState>) -> Result<Vec<WechatAccount>, String> {
    let db = lock_db!(state);
    db.get_all_accounts().map_err(|e| format!("获取微信账号失败: {}", e))
}

#[tauri::command]
pub fn add_account(state: State<AppState>, account: WechatAccount) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_account(&account).map_err(|e| format!("Failed to add account: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn update_account(state: State<AppState>, account: WechatAccount) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_account(&account).map_err(|e| format!("Failed to update account: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn delete_account(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_account(&id).map_err(|e| format!("Failed to delete account: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn save_wechat_account(state: State<AppState>, account: WechatAccount) -> Result<bool, String> {
    let db = lock_db!(state);

    if account.id.is_empty() {
        return Err("Account ID cannot be empty".to_string());
    }

    let existing = db.get_all_accounts().map_err(|e| format!("Failed to get accounts: {}", e))?;
    let exists = existing.iter().any(|a| a.id == account.id);

    if exists {
        db.update_account(&account).map_err(|e| format!("Failed to update account: {}", e)).and(Ok(true))
    } else {
        db.add_account(&account).map_err(|e| format!("Failed to add account: {}", e)).and(Ok(true))
    }
}

// ============ Verify WeChat ============
#[tauri::command]
pub async fn verify_wechat_account(app_id: String, app_secret: String) -> Result<String, String> {
    log::info!("[微信验证] 开始验证公众号 app_id={}", app_id);
    let client = crate::wechat::WechatClient::new(&app_id, &app_secret);
    match client.fetch_access_token().await {
        Ok(_) => {
            log::info!("[微信验证] 验证成功 app_id={}", app_id);
            Ok("验证成功，AppID 和 AppSecret 正确".to_string())
        }
        Err(e) => {
            log::error!("[微信验证] 验证失败 app_id={} error={}", app_id, e);
            Err(format!("验证失败: {}", e))
        }
    }
}
