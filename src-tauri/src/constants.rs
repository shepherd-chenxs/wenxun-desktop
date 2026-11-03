pub mod commands {
    pub const GREET: &str = "greet";
    pub const GET_TASKS: &str = "get_tasks";
    pub const ADD_TASK: &str = "add_task";
    pub const UPDATE_TASK: &str = "update_task";
    pub const DELETE_TASK: &str = "delete_task";
    pub const GET_ARTICLES: &str = "get_articles";
    pub const GET_ARTICLE: &str = "get_article";
    pub const SAVE_ARTICLE: &str = "save_article";
    pub const DELETE_ARTICLE: &str = "delete_article";
    pub const SAVE_ARTICLES_BATCH: &str = "save_articles_batch";
    pub const GET_TOPIC_CATEGORIES: &str = "get_topic_categories";
    pub const ADD_TOPIC_CATEGORY: &str = "add_topic_category";
    pub const UPDATE_TOPIC_CATEGORY: &str = "update_topic_category";
    pub const DELETE_TOPIC_CATEGORY: &str = "delete_topic_category";
    pub const GET_BLOGGERS: &str = "get_bloggers";
    pub const ADD_BLOGGER: &str = "add_blogger";
    pub const UPDATE_BLOGGER: &str = "update_blogger";
    pub const DELETE_BLOGGER: &str = "delete_blogger";
    pub const GET_ACCOUNTS: &str = "get_accounts";
    pub const ADD_ACCOUNT: &str = "add_account";
    pub const UPDATE_ACCOUNT: &str = "update_account";
    pub const DELETE_ACCOUNT: &str = "delete_account";
    pub const GET_TOPIC_PACKS: &str = "get_topic_packs";
    pub const ADD_TOPIC_PACK: &str = "add_topic_pack";
    pub const UPDATE_TOPIC_PACK: &str = "update_topic_pack";
    pub const DELETE_TOPIC_PACK: &str = "delete_topic_pack";
    pub const ADD_TOPIC_PACK_WITH_TOPICS: &str = "add_topic_pack_with_topics";
    pub const GET_AI_API_KEY: &str = "get_ai_api_key";
    pub const SAVE_AI_API_KEY: &str = "save_ai_api_key";
    pub const GET_AI_CONFIG: &str = "get_ai_config";
    pub const SAVE_AI_CONFIG: &str = "save_ai_config";
    pub const GET_TASK_LOGS: &str = "get_task_logs";
    pub const ADD_TASK_LOG: &str = "add_task_log";
    pub const CLEAR_TASK_LOGS: &str = "clear_task_logs";
    pub const GENERATE_ARTICLE: &str = "generate_article";
    pub const GENERATE_TITLE: &str = "generate_title";
    pub const GENERATE_SUMMARY: &str = "generate_summary";
    pub const WECHAT_PUBLISH: &str = "wechat_publish";
    pub const WECHAT_UPLOAD_MEDIA: &str = "wechat_upload_media";
    pub const GET_GENERATION_HISTORY: &str = "get_generation_history";
    pub const SAVE_GENERATION_HISTORY: &str = "save_generation_history";
    pub const CLEAR_GENERATION_HISTORY: &str = "clear_generation_history";
    pub const GET_TOPICS: &str = "get_topics";
    pub const ADD_TOPIC: &str = "add_topic";
    pub const UPDATE_TOPIC: &str = "update_topic";
    pub const DELETE_TOPIC: &str = "delete_topic";
    pub const GET_PUBLISH_HISTORY: &str = "get_publish_history";
    pub const ADD_PUBLISH_HISTORY: &str = "add_publish_history";
    pub const CLEAR_PUBLISH_HISTORY: &str = "clear_publish_history";
    pub const GET_PUBLIC_IP: &str = "get_public_ip";
    pub const GET_LOG_PATH: &str = "get_log_path";
}

pub mod table_names {
    pub const SCHEDULED_TASKS: &str = "scheduled_tasks";
    pub const ARTICLES: &str = "articles";
    pub const TOPIC_CATEGORIES: &str = "topic_categories";
    pub const VIRTUAL_BLOGGERS: &str = "virtual_bloggers";
    pub const WECHAT_ACCOUNTS: &str = "wechat_accounts";
    pub const WECHAT_ACCESS_TOKENS: &str = "wechat_access_tokens";
    pub const TOPIC_PACKS: &str = "topic_packs";
    pub const AI_API_KEYS: &str = "ai_api_keys";
    pub const AI_CONFIG: &str = "ai_config";
    pub const GENERATION_HISTORY: &str = "generation_history";
    pub const TOPICS: &str = "topics";
    pub const PUBLISH_HISTORY: &str = "publish_history";
    pub const TASK_EXECUTION_LOGS: &str = "task_execution_logs";
}

pub mod status {
    pub const DRAFT: &str = "draft";
    pub const PUBLISHED: &str = "published";
    pub const PENDING: &str = "pending";
    pub const FAILED: &str = "failed";
    pub const SUCCESS: &str = "success";
}

pub mod schedule_type {
    pub const DAILY: &str = "daily";
    pub const WEEKLY: &str = "weekly";
    pub const MONTHLY: &str = "monthly";
}