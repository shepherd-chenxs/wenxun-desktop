use tauri::State;
use crate::database::{Article, GenerationHistory, PublishHistory};
use crate::state::AppState;

// ============ Articles ============
#[tauri::command]
pub fn get_articles(state: State<AppState>) -> Result<Vec<Article>, String> {
    let db = lock_db!(state);
    db.get_all_articles().map_err(|e| format!("获取文章列表失败: {}", e))
}

#[tauri::command]
pub fn get_article(state: State<AppState>, id: String) -> Result<Option<Article>, String> {
    let db = lock_db!(state);
    db.get_article(&id).map_err(|e| format!("获取文章失败: {}", e))
}

#[tauri::command]
pub fn save_article(state: State<AppState>, article: Article) -> Result<bool, String> {
    let db = lock_db!(state);
    if db.get_article(&article.id).map_err(|e| format!("检查文章是否存在失败: {}", e))?.is_some() {
        db.update_article(&article).map_err(|e| format!("更新文章失败: {}", e))?;
        Ok(true)
    } else {
        db.add_article(&article).map_err(|e| format!("保存文章失败: {}", e))?;
        Ok(true)
    }
}

#[tauri::command]
pub fn delete_article(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_article(&id).map_err(|e| format!("Failed to delete article: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn save_articles_batch(state: State<AppState>, articles: Vec<Article>) -> Result<bool, String> {
    let db = lock_db!(state);
    db.save_articles_batch(&articles).map_err(|e| format!("Failed to batch save articles: {}", e)).and(Ok(true))
}

// ============ Generation History ============
#[tauri::command]
pub fn get_generation_history(state: State<AppState>) -> Result<Vec<GenerationHistory>, String> {
    let db = lock_db!(state);
    db.get_all_generation_history().map_err(|e| format!("获取生成历史失败: {}", e))
}

#[tauri::command]
pub fn save_generation_history(state: State<AppState>, history: GenerationHistory) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_generation_history(&history).map_err(|e| format!("Failed to save generation history: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn clear_generation_history(state: State<AppState>) -> Result<bool, String> {
    let db = lock_db!(state);
    db.clear_generation_history().map_err(|e| format!("Failed to clear generation history: {}", e)).and(Ok(true))
}

// ============ Publish History ============
#[tauri::command]
pub fn get_publish_history(state: State<AppState>) -> Result<Vec<PublishHistory>, String> {
    let db = lock_db!(state);
    db.get_all_publish_history().map_err(|e| format!("获取发布历史失败: {}", e))
}

#[tauri::command]
pub fn add_publish_history(state: State<AppState>, history: PublishHistory) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_publish_history(&history).map_err(|e| format!("Failed to add publish history: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn clear_publish_history(state: State<AppState>) -> Result<bool, String> {
    let db = lock_db!(state);
    db.clear_publish_history().map_err(|e| format!("Failed to clear publish history: {}", e)).and(Ok(true))
}
