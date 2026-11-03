#[cfg(test)]
mod tests {
    use crate::database::{Database, WechatAccessToken, AIConfig, TopicPack, Topic, VirtualBlogger};
    use std::fs;

    fn setup_test_db() -> Database {
        let test_db_path = "test_temp.db";
        let _ = fs::remove_file(test_db_path);
        let db = Database::new(test_db_path).expect("Failed to create test database");
        db
    }

    fn cleanup_test_db() {
        let _ = fs::remove_file("test_temp.db");
    }

    #[test]
    fn test_database_creation() {
        let db = setup_test_db();
        assert!(db.conn.lock().is_ok());
        cleanup_test_db();
    }

    #[test]
    fn test_init_tables() {
        let db = setup_test_db();
        let conn = db.conn.lock().unwrap();
        let result: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0)
        );
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
        drop(conn);
        cleanup_test_db();
    }

    #[test]
    fn test_wechat_access_token_crud() {
        let db = setup_test_db();
        let account_id = "test-account-1".to_string();
        let token = WechatAccessToken {
            account_id: account_id.clone(),
            access_token: "test_token_12345".to_string(),
            expires_at: 7200,
        };

        assert!(db.save_wechat_access_token(&token).is_ok());

        let stored = db.get_wechat_access_token(&account_id);
        assert!(stored.is_ok());
        let stored = stored.unwrap().unwrap();
        assert_eq!(stored.access_token, "test_token_12345");
        assert_eq!(stored.expires_at, 7200);

        let deleted = db.delete_wechat_access_token(&account_id);
        assert!(deleted.is_ok());

        let after_delete = db.get_wechat_access_token(&account_id);
        assert!(after_delete.is_ok());
        assert!(after_delete.unwrap().is_none());

        cleanup_test_db();
    }

    #[test]
    fn test_ai_config_crud() {
        let db = setup_test_db();
        let config = AIConfig {
            id: "config-1".to_string(),
            base_url: "https://api.test.com/v1".to_string(),
            model: "test-model".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        };

        assert!(db.save_ai_config(&config).is_ok());

        let stored = db.get_ai_config();
        assert!(stored.is_ok());
        let stored = stored.unwrap().unwrap();
        assert_eq!(stored.base_url, "https://api.test.com/v1");
        assert_eq!(stored.model, "test-model");
        assert_eq!(stored.temperature, 0.7);
        assert_eq!(stored.max_tokens, 1000);

        cleanup_test_db();
    }

    #[test]
    fn test_topic_pack_crud() {
        let db = setup_test_db();
        let pack = TopicPack {
            id: Some("pack-1".to_string()),
            name: "测试话题包".to_string(),
            category_id: "cat-1".to_string(),
            category_name: "技术".to_string(),
            description: Some("测试描述".to_string()),
            topics: "[]".to_string(),
            created_at: "1234567890".to_string(),
        };

        assert!(db.add_topic_pack(&pack).is_ok());

        let all_packs = db.get_all_topic_packs();
        assert!(all_packs.is_ok());
        assert!(all_packs.unwrap().iter().any(|p| p.name == "测试话题包"));

        cleanup_test_db();
    }

    #[test]
    fn test_topic_pack_with_topics_transaction() {
        let db = setup_test_db();
        let pack_id = "pack-trans-1".to_string();
        let pack = TopicPack {
            id: Some(pack_id.clone()),
            name: "事务测试包".to_string(),
            category_id: "cat-1".to_string(),
            category_name: "技术".to_string(),
            description: None,
            topics: "[]".to_string(),
            created_at: "1234567890".to_string(),
        };

        let topics = vec![
            Topic {
                id: Some("topic-1".to_string()),
                pack_id: pack_id.clone(),
                content: "话题1内容".to_string(),
                status: "active".to_string(),
                created_at: "1234567890".to_string(),
            },
            Topic {
                id: Some("topic-2".to_string()),
                pack_id: pack_id.clone(),
                content: "话题2内容".to_string(),
                status: "active".to_string(),
                created_at: "1234567891".to_string(),
            },
        ];

        assert!(db.add_topic_pack_with_topics(&pack, &topics).is_ok());

        let stored_topics = db.get_topics_by_pack(Some(&pack_id));
        assert!(stored_topics.is_ok());
        assert!(stored_topics.unwrap().len() >= 1);

        cleanup_test_db();
    }

    #[test]
    fn test_topic_pack_delete_with_topics() {
        let db = setup_test_db();
        let pack_id = "pack-del-1".to_string();
        let pack = TopicPack {
            id: Some(pack_id.clone()),
            name: "删除测试包".to_string(),
            category_id: "cat-1".to_string(),
            category_name: "技术".to_string(),
            description: None,
            topics: "[]".to_string(),
            created_at: "1234567890".to_string(),
        };

        let topics = vec![
            Topic {
                id: Some("topic-del-1".to_string()),
                pack_id: pack_id.clone(),
                content: "待删话题".to_string(),
                status: "available".to_string(),
                created_at: "1234567890".to_string(),
            },
        ];

        assert!(db.add_topic_pack_with_topics(&pack, &topics).is_ok());
        assert!(db.delete_topic_pack_with_topics(&pack_id).is_ok());

        let stored_topics = db.get_topics_by_pack(Some(&pack_id));
        assert!(stored_topics.is_ok());
        assert_eq!(stored_topics.unwrap().len(), 0);

        cleanup_test_db();
    }

    #[test]
    fn test_save_articles_batch() {
        let db = setup_test_db();
        let articles = vec![
            crate::database::Article {
                id: "batch-1".to_string(),
                title: "批量文章1".to_string(),
                content: "内容1".to_string(),
                cover_media_id: None,
                author_name: None,
                category: None,
                tags: None,
                status: "draft".to_string(),
                created_at: "1234567890".to_string(),
                updated_at: "1234567890".to_string(),
            quality_report: None,
            },
            crate::database::Article {
                id: "batch-2".to_string(),
                title: "批量文章2".to_string(),
                content: "内容2".to_string(),
                cover_media_id: None,
                author_name: None,
                category: None,
                tags: None,
                status: "draft".to_string(),
                created_at: "1234567891".to_string(),
                updated_at: "1234567891".to_string(),
            quality_report: None,
            },
        ];

        assert!(db.save_articles_batch(&articles).is_ok());

        let stored = db.get_article("batch-1");
        assert!(stored.is_ok());
        assert_eq!(stored.unwrap().unwrap().title, "批量文章1");

        let stored2 = db.get_article("batch-2");
        assert!(stored2.is_ok());
        assert_eq!(stored2.unwrap().unwrap().title, "批量文章2");

        cleanup_test_db();
    }

    #[test]
    fn test_virtual_blogger_crud() {
        let db = setup_test_db();
        let blogger = VirtualBlogger {
            id: "blogger-full".to_string(),
            name: "完整博主".to_string(),
            persona: "技术专家".to_string(),
            topic_category: "技术".to_string(),
            topic_category_id: "cat-1".to_string(),
            topic_category_name: "技术分类".to_string(),
            writing_style: "专业、严谨".to_string(),
            language_style: "简洁明了".to_string(),
            article_structure: "标准格式".to_string(),
            layout_format: "图文并茂".to_string(),
            tone_keywords: "专业、严谨".to_string(),
            content_bottom_line: "不发布虚假信息".to_string(),
            default_word_count: 2000,
            is_builtin: false,
            preset_key: String::new(),
            lessons: String::new(),
        };

        assert!(db.add_blogger(&blogger).is_ok());

        let all_bloggers = db.get_all_bloggers();
        assert!(all_bloggers.is_ok());
        assert!(all_bloggers.unwrap().len() >= 1);

        cleanup_test_db();
    }

    #[test]
    fn test_article_crud() {
        let db = setup_test_db();
        let article = crate::database::Article {
            id: "article-test-1".to_string(),
            title: "测试文章".to_string(),
            content: "测试内容".to_string(),
            cover_media_id: None,
            author_name: None,
            category: None,
            tags: None,
            status: "draft".to_string(),
            created_at: "1234567890".to_string(),
            updated_at: "1234567890".to_string(),
        quality_report: None,
        };

        assert!(db.add_article(&article).is_ok());

        let stored = db.get_article("article-test-1");
        assert!(stored.is_ok());
        assert_eq!(stored.unwrap().unwrap().title, "测试文章");

        cleanup_test_db();
    }

    #[test]
    fn test_topic_category_crud() {
        let db = setup_test_db();
        let category = crate::database::TopicCategory {
            id: "cat-test-1".to_string(),
            name: "测试分类".to_string(),
            description: Some("测试描述".to_string()),
            created_at: "1234567890".to_string(),
        };

        assert!(db.add_topic_category(&category).is_ok());

        let all_categories = db.get_all_topic_categories();
        assert!(all_categories.is_ok());
        assert!(all_categories.unwrap().iter().any(|c| c.name == "测试分类"));

        cleanup_test_db();
    }
}