use tauri::State;
use crate::database::{ScheduledTask, Article, TaskExecutionLog};
use crate::state::AppState;

// ============ Scheduled Tasks ============
#[tauri::command]
pub fn get_tasks(state: State<AppState>) -> Result<Vec<ScheduledTask>, String> {
    let db = lock_db!(state);
    db.get_all_tasks().map_err(|e| format!("获取任务列表失败: {}", e))
}

#[tauri::command]
pub fn add_task(state: State<AppState>, mut task: ScheduledTask) -> Result<bool, String> {
    let db = lock_db!(state);
    
    // 参数校验
    if task.name.is_empty() || task.name.len() > 100 {
        return Err("任务名称长度必须在 1-100 字符之间".to_string());
    }
    if task.schedule_type.is_empty() {
        return Err("调度类型不能为空".to_string());
    }
    if task.publish_time.is_empty() {
        return Err("发布时间不能为空".to_string());
    }
    if task.blogger_id.is_empty() {
        return Err("博主ID不能为空".to_string());
    }
    if task.account_id.is_empty() {
        return Err("公众号账号ID不能为空".to_string());
    }
    if task.word_count < 100 || task.word_count > 10000 {
        return Err("字数必须在 100-10000 之间".to_string());
    }
    
    // 如果没有 ID，生成一个新的 ID
    if task.id.is_none() {
        task.id = Some(format!("{}", chrono::Utc::now().timestamp_millis()));
    }
    
    println!("Adding task: {:?}", task.id);
    match db.add_task(&task) {
        Ok(_) => {
            println!("Task added successfully: {:?}", task.id);
            Ok(true)
        },
        Err(e) => {
            println!("Failed to add task: {}", e);
            Err(format!("Failed to add task: {}", e))
        }
    }
}

#[tauri::command]
pub fn update_task(state: State<AppState>, task: ScheduledTask) -> Result<bool, String> {
    let db = lock_db!(state);
    db.update_task(&task).map_err(|e| format!("Failed to update task: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_task(&id).map_err(|e| format!("Failed to delete task: {}", e)).and(Ok(true))
}

#[tauri::command]
pub async fn execute_task(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let start_time = chrono::Utc::now();
    log::info!("[任务执行] 开始执行任务 id={}", id);
    let mut task_name = String::new();
    let mut log_topic: Option<String> = None;
    let mut log_article_title: Option<String> = None;
    let mut log_status = "success";
    let mut log_publish_status: Option<String> = None;
    let mut log_result: Option<String> = None;
    let mut log_error: Option<String> = None;

    let result = {
        let state_ref = &state;
        let future = run_task_inner(state_ref, &id, &mut task_name, &mut log_topic, &mut log_article_title);
        match tokio::time::timeout(std::time::Duration::from_secs(300), future).await {
            Ok(inner) => inner,
            Err(_) => {
                log::error!("[任务执行] 任务执行超时（5分钟）");
                Err("任务执行超时（5分钟）".to_string())
            }
        }
    };

    match &result {
        Ok(msg) => {
            log_result = Some(msg.clone());
            // 从返回信息中提取发布状态和错误
            if msg.contains("发布失败") || msg.contains("提交发布失败") || msg.contains("创建草稿失败") {
                log_publish_status = Some("failed".to_string());
                log_error = Some(msg.clone());
            } else if msg.contains("微信") || msg.contains("草稿已保存") {
                log_publish_status = Some("success".to_string());
            }
        }
        Err(e) => {
            log_status = "failed";
            log_error = Some(e.clone());
            // 检查是否在生成文章后才失败（发布阶段失败）
            if e.contains("发布失败") || e.contains("提交发布失败") || e.contains("创建草稿失败") {
                log_publish_status = Some("failed".to_string());
            }
        }
    }

    let log_entry = TaskExecutionLog {
        id: format!("{}", chrono::Utc::now().timestamp_millis()),
        task_id: id,
        task_name,
        execution_time: start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        status: log_status.to_string(),
        publish_status: log_publish_status,
        topic: log_topic,
        article_title: log_article_title,
        duration: Some((chrono::Utc::now() - start_time).num_seconds() as i32),
        result: log_result,
        error_message: log_error,
    };

    {
        let db = lock_db!(state);
        if let Err(e) = db.add_task_log(&log_entry) {
            log::error!("[任务执行] 保存执行日志失败: {}", e);
        }
    }

    let task_name_for_log = &log_entry.task_name;
    match &result {
        Ok(_) => log::info!("[任务执行] 任务 {} 执行成功", task_name_for_log),
        Err(e) => log::error!("[任务执行] 任务 {} 执行失败: {}", task_name_for_log, e),
    }

    result
}

async fn run_task_inner(
    state: &State<'_, AppState>,
    id: &str,
    task_name: &mut String,
    log_topic: &mut Option<String>,
    log_article_title: &mut Option<String>,
) -> Result<String, String> {
    log::info!("[run_task_inner] 开始获取任务信息");
    let task = {
        let db = lock_db!(state);
        db.get_task(id).map_err(|e| format!("获取任务失败: {}", e))?
            .ok_or_else(|| format!("任务不存在: {}", id))?
    };
    *task_name = task.name.clone();
    log::info!("[run_task_inner] 任务: {}, publish_type: {}", task.name, task.publish_type);

    log::info!("[run_task_inner] 获取 AI 配置");
    let (api_key, base_url, model, temperature, max_tokens) = {
        let db = lock_db!(state);
        let api_key = match db.get_ai_api_key().map_err(|e| format!("获取 API Key 失败: {}", e))? {
            Some(key) => key.api_key,
            None => return Err("AI API Key 未配置".to_string()),
        };
        let config = db.get_ai_config().map_err(|e| format!("获取 AI 配置失败: {}", e))?;
        let base_url = config.as_ref().map(|c| c.base_url.clone()).unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
        let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_else(|| "deepseek-chat".to_string());
        let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
        let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
        (api_key, base_url, model, temperature, max_tokens)
    };
    log::info!("[run_task_inner] AI 配置获取成功, model={}", model);

    log::info!("[run_task_inner] 开始生成话题");
    let topic = if task.topic_source == "ai" {
        let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
        let prompt = format!("请为题材\"{}\"生成一个吸引人的文章话题，只需要返回话题本身，不要有其他说明文字。", task.topic_category_name);
        client.generate_article(&prompt, 50, "").await?
    } else if task.topic_source == "hot" {
        let topic = crate::hot::pick_hot_topic(&task.hot_source)
            .await
            .map_err(|e| format!("获取热点话题失败: {}", e))?;
        log::info!("[run_task_inner] 使用热点雷达选题: {}", topic);
        topic
    } else {
        let db = lock_db!(state);
        if let Some(ref pack_id) = task.topic_pack_id {
            let topics = db.get_topics_by_pack(Some(pack_id)).map_err(|e| format!("获取话题失败: {}", e))?;
            if topics.is_empty() {
                return Err("话题包中没有话题".to_string());
            }
            let available_topics: Vec<_> = topics.iter().filter(|t| t.status == "available").collect();
            if available_topics.is_empty() {
                return Err("话题包中没有可用话题".to_string());
            }
            let topic_content = available_topics[0].content.clone();
            let mut used_topic = available_topics[0].clone();
            used_topic.status = "used".to_string();
            db.update_topic(&used_topic).map_err(|e| format!("更新话题状态失败: {}", e))?;
            topic_content
        } else {
            return Err("话题来源为话题包时，必须指定话题包ID".to_string());
        }
    };
    *log_topic = Some(topic.clone());
    log::info!("[run_task_inner] 话题: {}", topic);

    log::info!("[run_task_inner] 开始生成文章, word_count={}", task.word_count);
    let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
    let sys = "你是顶尖的公众号写手。输出要求：第一行是文章标题（不带书名号），随后空一行输出正文 Markdown。";
    let usr = format!("请围绕话题《{}》写一篇约 {} 字的原创文章。", topic, task.word_count);
    let outcome = crate::quality::review_and_polish(&client, sys, &usr, 2).await?;
    let content = outcome.content;
    {
        let persona_name = {
            let db2 = lock_db!(state);
            db2.get_task(&id).ok().flatten().and_then(|t| db2.get_blogger(&t.blogger_id).ok().flatten().map(|b| b.name))
        };
        let db2 = lock_db!(state);
        db2.record_quality_report(None, "", "task", persona_name.as_deref(), outcome.score, &outcome.report);
    }
    log::info!("[run_task_inner] 文章生成成功, 长度={}", content.len());

    log::info!("[run_task_inner] 开始生成标题");
    let title = client.generate_title(&content).await
        .map_err(|e| format!("生成标题失败: {}", e))?;
    let first_title = crate::ai::clean_title(&title, &topic);
    log::info!("[run_task_inner] 标题: {}", first_title);

    let article_id = format!("{}", chrono::Utc::now().timestamp_millis());
    let article = Article {
        id: article_id.clone(),
        title: first_title.clone(),
        content: content.clone(),
        cover_media_id: None,
        author_name: None,
        category: None,
        tags: None,
        status: "draft".to_string(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        quality_report: Some(outcome.report.clone()),
    };
    *log_article_title = Some(article.title.clone());

    log::info!("[run_task_inner] 保存文章到数据库");
    {
        let db = lock_db!(state);
        db.add_article(&article).map_err(|e| format!("保存文章失败: {}", e))?;
    }
    log::info!("[run_task_inner] 文章已保存, id={}", article_id);

    let mut publish_result = String::new();

    if !task.account_id.is_empty() {
        log::info!("[run_task_inner] 开始发布到微信公众号, publish_type={}", task.publish_type);
        let account = {
            let db = lock_db!(state);
            let accounts = db.get_all_accounts().map_err(|e| format!("获取微信账户列表失败: {}", e))?;
            accounts.into_iter()
                .find(|a| a.id == task.account_id)
                .ok_or_else(|| format!("公众号账号不存在: {}", task.account_id))?
        };
        log::info!("[run_task_inner] 公众号: {}", account.name);

        let wechat_client = crate::wechat::WechatClient::new(&account.app_id, &account.app_secret);
        log::info!("[run_task_inner] 获取 access_token...");
        let token_data = wechat_client.fetch_access_token().await
            .map_err(|e| format!("获取 access_token 失败: {}", e))?;
        log::info!("[run_task_inner] access_token 获取成功");

        let cover_media_id = if !task.cover_media_id.trim().is_empty() {
            Some(task.cover_media_id.as_str())
        } else if !account.default_cover_media_id.trim().is_empty() {
            Some(account.default_cover_media_id.as_str())
        } else {
            None
        };

        log::info!("[run_task_inner] 创建草稿...");
        let wechat_content = crate::ai::text_to_wechat_html(&article.content);
        let draft_result = wechat_client.create_draft(
            &token_data.access_token,
            &article.title,
            &wechat_content,
            task.author_name.as_deref(),
            cover_media_id,
        ).await
        .map_err(|e| format!("创建草稿失败: {}", e))?;
        log::info!("[run_task_inner] 草稿创建结果: errcode={}", draft_result.errcode);

        if draft_result.errcode != 0 {
            publish_result = format!("，创建草稿失败: {} (错误码: {})",
                draft_result.errmsg, draft_result.errcode);

            let publish_history = crate::database::PublishHistory {
                id: format!("{}", chrono::Utc::now().timestamp_millis()),
                article_id: article_id.clone(),
                article_title: article.title.clone(),
                account_id: account.id.clone(),
                publish_time: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                status: "failed".to_string(),
            };

            let db = lock_db!(state);
            db.add_publish_history(&publish_history)
                .map_err(|e| format!("记录发布历史失败: {}", e))?;
        } else {
            let is_mass = task.publish_type == "mass";

            if is_mass {
                let media_id = draft_result.media_id.as_deref().unwrap_or("");
                if !media_id.is_empty() {
                    match wechat_client.publish_draft(&token_data.access_token, media_id).await {
                        Ok(submit_result) if submit_result.errcode == 0 => {
                            publish_result = format!("，已发布到微信公众号: {}", account.name);
                        }
                        Ok(submit_result) => {
                            publish_result = format!("，提交发布失败: {} (错误码: {})", submit_result.errmsg, submit_result.errcode);
                        }
                        Err(e) => {
                            publish_result = format!("，提交发布失败: {}", e);
                        }
                    }
                }
            } else {
                publish_result = format!("，草稿已保存至微信公众号");
            }

            let hist_status = if publish_result.contains("失败") { "failed" } else { if is_mass { "success" } else { "draft" } };
            let publish_history = crate::database::PublishHistory {
                id: format!("{}", chrono::Utc::now().timestamp_millis()),
                article_id: article_id.clone(),
                article_title: article.title.clone(),
                account_id: account.id.clone(),
                publish_time: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                status: hist_status.to_string(),
            };

            let should_mark_published = (is_mass && hist_status == "success") || (!is_mass && hist_status == "draft");
            let mut updated_article = article.clone();
            if should_mark_published {
                updated_article.status = "published".to_string();
            }

            let db = lock_db!(state);
            db.add_publish_history(&publish_history)
                .map_err(|e| format!("记录发布历史失败: {}", e))?;
            if should_mark_published {
                db.update_article(&updated_article)
                    .map_err(|e| format!("更新文章状态失败: {}", e))?;
            }
        }
    }

    let result_msg = format!("生成文章: {}{}", article.title, publish_result);
    if publish_result.contains("失败") {
        // ⚠️ 发布阶段失败，但文章已经生成并保存成功，不掩盖生成的成功
        // 日志中的 publish_status 字段会记录发布失败详情
        log::warn!("[任务执行] 文章生成成功但发布失败: {}", result_msg);
    }
    log::info!("[run_task_inner] 完成, 返回: {}", result_msg);
    Ok(result_msg)
}

// ============ Task Execution Logs ============
#[tauri::command]
pub fn get_task_logs(state: State<AppState>, task_id: Option<String>) -> Result<Vec<TaskExecutionLog>, String> {
    let db = lock_db!(state);
    db.get_task_logs(task_id.as_deref()).map_err(|e| format!("获取任务日志失败: {}", e))
}

#[tauri::command]
pub fn add_task_log(state: State<AppState>, log: TaskExecutionLog) -> Result<bool, String> {
    let db = lock_db!(state);
    db.add_task_log(&log).map_err(|e| format!("Failed to add task log: {}", e)).and(Ok(true))
}

#[tauri::command]
pub fn clear_task_logs(state: State<AppState>, task_id: Option<String>) -> Result<bool, String> {
    let db = lock_db!(state);
    db.clear_task_logs(task_id.as_deref()).map_err(|e| format!("Failed to clear task logs: {}", e)).and(Ok(true))
}
