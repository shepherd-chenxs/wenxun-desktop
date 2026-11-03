use tauri::State;
use crate::database::VirtualBlogger;
use crate::state::AppState;

// ============ Virtual Bloggers ============
#[tauri::command]
pub fn get_bloggers(state: State<AppState>) -> Result<Vec<VirtualBlogger>, String> {
    let db = lock_db!(state);
    db.get_all_bloggers().map_err(|e| format!("获取虚拟博主失败: {}", e))
}

#[tauri::command]
pub fn add_blogger(state: State<AppState>, blogger: VirtualBlogger) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_blogger(&blogger).map_err(|e| format!("Failed to add blogger: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn update_blogger(state: State<AppState>, blogger: VirtualBlogger) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_blogger(&blogger).map_err(|e| format!("Failed to update blogger: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn save_blogger(state: State<AppState>, blogger: VirtualBlogger) -> Result<bool, String> {
    let db = lock_db!(state);
    if db.get_blogger(&blogger.id).map_err(|e| format!("检查博主失败: {}", e))?.is_some() {
        db.update_blogger(&blogger).map_err(|e| format!("更新博主失败: {}", e))?;
    } else {
        db.add_blogger(&blogger).map_err(|e| format!("添加博主失败: {}", e))?;
    }
    Ok(true)
}

#[tauri::command]
pub fn delete_blogger(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_blogger(&id).map_err(|e| format!("Failed to delete blogger: {}", e)).and(Ok(true))
}
