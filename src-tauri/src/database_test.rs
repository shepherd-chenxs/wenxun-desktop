#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_db() -> Database {
        let test_db_path = "test_temp.db";
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
        // 验证表是否创建成功
        let conn = db.conn.lock().unwrap();
        let result: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0)
        );
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
        cleanup_test_db();
    }

    #[test]
    fn test_topic_category_crud() {
        let db = setup_test_db();
        
        // 创建题材分类
        let id = "test-category-1".to_string();
        let name = "测试分类".to_string();
        let description = Some("这是一个测试分类".to_string());
        
        let result = db.create_topic_category(&id, &name, &description);
        assert!(result.is_ok());
        
        // 查询题材分类
        let category = db.get_topic_category(&id);
        assert!(category.is_ok());
        let category = category.unwrap();
        assert_eq!(category.name, name);
        
        // 更新题材分类
        let new_name = "更新后的分类".to_string();
        let update_result = db.update_topic_category(&id, &new_name, &description);
        assert!(update_result.is_ok());
        
        // 删除题材分类
        let delete_result = db.delete_topic_category(&id);
        assert!(delete_result.is_ok());
        
        cleanup_test_db();
    }

    #[test]
    fn test_virtual_blogger_crud() {
        let db = setup_test_db();
        
        // 先创建一个题材分类
        let category_id = "test-category-2".to_string();
        let _ = db.create_topic_category(&category_id, "测试分类", &None);
        
        // 创建虚拟博主
        let blogger_id = "test-blogger-1".to_string();
        let result = db.create_virtual_blogger(
            &blogger_id,
            "测试博主",
            "技术专家",
            &category_id,
            "专业、深入",
            "简洁明了",
            "逻辑清晰",
            "标准格式",
            "专业、严谨",
            "不发布虚假信息",
            2000,
        );
        assert!(result.is_ok());
        
        // 查询虚拟博主
        let blogger = db.get_virtual_blogger(&blogger_id);
        assert!(blogger.is_ok());
        
        cleanup_test_db();
    }

    #[test]
    fn test_article_crud() {
        let db = setup_test_db();
        
        // 创建文章
        let article_id = "test-article-1".to_string();
        let result = db.create_article(
            &article_id,
            "测试文章标题",
            "测试文章内容",
            None,
            None,
            None,
            None,
            "draft",
        );
        assert!(result.is_ok());
        
        // 查询文章
        let article = db.get_article(&article_id);
        assert!(article.is_ok());
        let article = article.unwrap();
        assert_eq!(article.title, "测试文章标题");
        
        // 更新文章状态
        let update_result = db.update_article_status(&article_id, "published");
        assert!(update_result.is_ok());
        
        cleanup_test_db();
    }
}
