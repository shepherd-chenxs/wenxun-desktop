use tauri::State;
use crate::database::{TopicCategory, TopicPack, Topic};
use crate::state::AppState;

// ============ Topic Categories ============
#[tauri::command]
pub fn get_topic_categories(state: State<AppState>) -> Result<Vec<TopicCategory>, String> {
    let db = lock_db!(state);
    db.get_all_topic_categories().map_err(|e| format!("获取题材分类失败: {}", e))
}

#[tauri::command]
pub fn add_topic_category(state: State<AppState>, category: TopicCategory) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_topic_category(&category).map_err(|e| format!("Failed to add category: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn update_topic_category(state: State<AppState>, category: TopicCategory) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_topic_category(&category).map_err(|e| format!("Failed to update category: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn delete_topic_category(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_topic_category(&id).map_err(|e| format!("Failed to delete category: {}", e)).and(Ok(true))
}

// ============ Topic Packs ============
#[tauri::command]
pub fn get_topic_packs(state: State<AppState>) -> Result<Vec<TopicPack>, String> {
    let db = lock_db!(state);
    db.get_all_topic_packs().map_err(|e| format!("获取话题包失败: {}", e))
}

#[tauri::command]
pub fn add_topic_pack(state: State<AppState>, mut pack: TopicPack) -> Result<bool, String> {
    let db = lock_db!(state);

    if pack.id.is_none() {
        pack.id = Some(format!("{}", chrono::Utc::now().timestamp_millis()));
    }

    db.add_topic_pack(&pack).map_err(|e| format!("Failed to add topic pack: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn update_topic_pack(state: State<AppState>, pack: TopicPack) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_topic_pack(&pack).map_err(|e| format!("Failed to update topic pack: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn delete_topic_pack(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_topic_pack_with_topics(&id).map_err(|e| format!("Failed to delete topic pack: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn save_topic_pack(state: State<AppState>, pack: TopicPack) -> Result<bool, String> {
    let db = lock_db!(state);

    if pack.id.is_none() || pack.id.as_ref().map(|id| id.is_empty()).unwrap_or(true) {
        let mut new_pack = pack;
        if new_pack.id.is_none() || new_pack.id.as_ref().map(|id| id.is_empty()).unwrap_or(true) {
            new_pack.id = Some(format!("{}", chrono::Utc::now().timestamp_millis()));
        }
        db.add_topic_pack(&new_pack).map_err(|e| format!("Failed to add topic pack: {}", e)).and(Ok(true))
    } else {
        db.update_topic_pack(&pack).map_err(|e| format!("Failed to update topic pack: {}", e)).and(Ok(true))
    }
}

#[tauri::command]
pub fn add_topic_pack_with_topics(state: State<AppState>, mut pack: TopicPack, topics: Vec<Topic>) -> Result<bool, String> {
    let db = lock_db!(state);

    if pack.id.is_none() {
        pack.id = Some(format!("{}", chrono::Utc::now().timestamp_millis()));
    }

    db.add_topic_pack_with_topics(&pack, &topics).map_err(|e| format!("Failed to add topic pack with topics: {}", e)).and(Ok(true))
}

// ============ Topics ============
#[tauri::command]
pub fn get_topics(state: State<AppState>, pack_id: Option<String>) -> Result<Vec<Topic>, String> {
    let db = lock_db!(state);
    db.get_topics_by_pack(pack_id.as_deref()).map_err(|e| format!("获取话题失败: {}", e))
}

#[tauri::command]
pub fn add_topic(state: State<AppState>, mut topic: Topic) -> Result<bool, String> {
    let db = lock_db!(state);

    if topic.id.is_none() {
        topic.id = Some(format!("{}", chrono::Utc::now().timestamp_millis()));
    }

    db.add_topic(&topic).map_err(|e| format!("Failed to add topic: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn update_topic(state: State<AppState>, topic: Topic) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_topic(&topic).map_err(|e| format!("Failed to update topic: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn delete_topic(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_topic(&id).map_err(|e| format!("Failed to delete topic: {}", e)).and(Ok(true))
}
