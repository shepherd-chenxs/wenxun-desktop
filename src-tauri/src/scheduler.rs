use std::time::Duration;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Datelike};
use tokio::time::{sleep, timeout};
use tokio::sync::{Semaphore, Mutex as TokioMutex};
use crate::database::Database;

#[derive(Clone)]
pub struct TaskScheduler {
    db: Arc<Mutex<Database>>,
    running: Arc<TokioMutex<bool>>,
    semaphore: Arc<Semaphore>,
    token_cache: Arc<TokioMutex<std::collections::HashMap<String, (String, i64)>>>,
}

impl TaskScheduler {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            running: Arc::new(TokioMutex::new(false)),
            semaphore: Arc::new(Semaphore::new(3)),
            token_cache: Arc::new(TokioMutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            log::info!("[scheduler] 调度器已经在运行中");
            return;
        }
        *running = true;
        drop(running);

        // 启动时恢复：将上次崩溃残留的 running 状态重置为 pending
        {
            let db = self.db.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(tasks) = db.get_all_tasks() {
                for mut task in tasks {
                    if task.status == "running" {
                        task.status = "pending".to_string();
                        let _ = db.update_task(&task);
                        log::info!("[scheduler] 恢复任务状态: {} -> pending", task.name);
                    }
                }
            }
        }

        log::info!("[scheduler] 定时任务调度器已启动");

        loop {
            let should_stop = {
                let running = self.running.lock().await;
                !*running
            };
            
            if should_stop {
                log::info!("[scheduler] 调度器已停止");
                break;
            }

            match self.check_and_execute_tasks().await {
                Ok(executed_count) => {
                    if executed_count > 0 {
                        log::info!("[scheduler] 本次检查执行了 {} 个任务", executed_count);
                    }
                }
                Err(e) => {
                    log::info!("[scheduler] 检查任务时出错: {}", e);
                }
            }

            sleep(Duration::from_secs(60)).await;
        }
    }

    pub fn stop(&self) {
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.block_on(async {
                let mut running = self.running.lock().await;
                *running = false;
            });
            log::info!("[scheduler] 正在停止调度器...");
        } else {
            log::warn!("[scheduler] 无法获取 Tokio runtime，调度器可能已在停止中");
        }
    }

    async fn check_and_execute_tasks(&self) -> Result<usize, String> {
        let now = Utc::now();
        
        log::info!("[scheduler] 开始检查定时任务");

        let tasks = {
            let db = self.db.lock().unwrap_or_else(|e| e.into_inner());
            db.get_all_tasks().map_err(|e| format!("获取任务列表失败: {}", e))?
        };

        let mut executed_count = 0;

        for task in tasks {
            if !task.enabled {
                continue;
            }

            if self.should_execute_task(&task, &now) {
                log::info!("[scheduler] 任务 '{}' 到达执行时间", task.name);

                // 先标记 running，防止下一轮检查重复触发
                {
                    let mut t = task.clone();
                    t.status = "running".to_string();
                    let db = self.db.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = db.update_task(&t);
                }

                let task_id = task.id.clone().unwrap_or_default();
                let db_clone = Arc::clone(&self.db);
                let sem_clone = Arc::clone(&self.semaphore);
                let token_cache = Arc::clone(&self.token_cache);

                tokio::spawn(async move {
                    let _permit = match sem_clone.acquire().await {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("[scheduler] 获取信号量失败: {}", e);
                            return;
                        }
                    };
                    match Self::execute_scheduled_task(db_clone, task_id, token_cache).await {
                        Ok(_) => log::info!("[scheduler] 任务执行成功"),
                        Err(e) => log::error!("[scheduler] 任务执行失败: {}", e),
                    }
                });

                executed_count += 1;

                if task.schedule_type != "once" {
                    self.update_next_execution_time(&task).await?;
                }
            }
        }

        Ok(executed_count)
    }

    fn should_execute_task(&self, task: &crate::database::ScheduledTask, now: &DateTime<Utc>) -> bool {
        let publish_time = match chrono::DateTime::parse_from_rfc3339(&task.publish_time) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                log::info!("[scheduler] 任务 '{}' 的时间格式错误", task.name);
                return false;
            }
        };

        match task.schedule_type.as_str() {
            "once" => now >= &publish_time && task.status == "pending",
            "daily" => {
                let diff = now.timestamp() - publish_time.timestamp();
                diff >= 0 && diff <= 60 && task.status != "running"
            }
            "weekly" => {
                if let Some(week_day) = task.week_day {
                    let now_weekday = now.weekday().num_days_from_sunday();
                    if now_weekday != week_day as u32 {
                        return false;
                    }
                    let diff = now.timestamp() - publish_time.timestamp();
                    diff >= 0 && diff <= 60 && task.status != "running"
                } else {
                    false
                }
            }
            "monthly" => {
                if let Some(month_day) = task.month_day {
                    if now.day() != month_day as u32 {
                        return false;
                    }
                    let diff = now.timestamp() - publish_time.timestamp();
                    diff >= 0 && diff <= 60 && task.status != "running"
                } else {
                    false
                }
            }
            _ => {
                log::info!("[scheduler] 未知的调度类型: {}", task.schedule_type);
                false
            }
        }
    }

    async fn execute_scheduled_task(
        db: Arc<Mutex<Database>>, 
        task_id: String,
        token_cache: Arc<TokioMutex<std::collections::HashMap<String, (String, i64)>>>,
    ) -> Result<(), String> {
        use std::time::Instant;
        let start_time = Instant::now();
        let execution_start_time = chrono::Utc::now();  // 记录任务开始时间
        
        log::info!("[scheduler] 开始执行定时任务: {}", task_id);
        
        let task = {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.get_task(&task_id)
                .map_err(|e| format!("获取任务失败: {}", e))?
                .ok_or_else(|| format!("任务不存在: {}", task_id))?
        };
        
        {
            let mut running_task = task.clone();
            running_task.status = "running".to_string();
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.update_task(&running_task)
                .map_err(|e| format!("更新任务状态失败: {}", e))?;
        }
        
        let result = match timeout(Duration::from_secs(300), Self::execute_task_inner(Arc::clone(&db), &task, Arc::clone(&token_cache))).await {
            Ok(inner_result) => inner_result,
            Err(_) => {
                log::info!("[scheduler] 任务执行超时（5分钟）");
                Err("任务执行超时".to_string())
            }
        };
        
        let duration = start_time.elapsed().as_secs() as i32;

        // ⚠️ 写日志和更新任务状态需要分别加锁，避免 tokio::Mutex 双重加锁死锁
        {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            match &result {
                Ok((topic, title, publish_status)) => {
                    let log_entry = crate::database::TaskExecutionLog {
                        id: format!("{}", chrono::Utc::now().timestamp_millis()),
                        task_id: task_id.clone(),
                        task_name: task.name.clone(),
                        execution_time: execution_start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        status: "success".to_string(),
                        publish_status: Some(publish_status.clone()),
                        topic: Some(topic.clone()),
                        article_title: Some(title.clone()),
                        duration: Some(duration),
                        result: Some(format!("文章创建成功: {}", title)),
                        error_message: None,
                    };
                    db_lock.add_task_log(&log_entry)
                        .map_err(|e| format!("添加执行日志失败: {}", e))?;
                    log::info!("[scheduler] 任务执行成功: {}, 耗时: {}秒, 发布状态: {}", title, duration, publish_status);
                }
                Err(e) => {
                    let log_entry = crate::database::TaskExecutionLog {
                        id: format!("{}", chrono::Utc::now().timestamp_millis()),
                        task_id: task_id.clone(),
                        task_name: task.name.clone(),
                        execution_time: execution_start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        status: "failed".to_string(),
                        publish_status: None,
                        topic: None,
                        article_title: None,
                        duration: Some(duration),
                        result: None,
                        error_message: Some(e.clone()),
                    };
                    db_lock.add_task_log(&log_entry)
                        .map_err(|e| format!("添加执行日志失败: {}", e))?;
                    log::info!("[scheduler] 任务执行失败: {}, 错误: {}", task.name, e);
                }
            }
        } // db_lock 在此释放

        {
            let mut final_task = task.clone();
            final_task.status = if result.is_ok() {
                if task.schedule_type == "once" { "completed".to_string() } else { "pending".to_string() }
            } else {
                "failed".to_string()
            };
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.update_task(&final_task)
                .map_err(|e| format!("更新任务状态失败: {}", e))?;
        }

        result.map(|_| ()) // 忽略 (topic, title, publish_status) 元组细节
    }
    
    async fn execute_task_inner(
        db: Arc<Mutex<Database>>,
        task: &crate::database::ScheduledTask,
        token_cache: Arc<TokioMutex<std::collections::HashMap<String, (String, i64)>>>,
    ) -> Result<(String, String, String), String> {
        // 1. 获取话题
        let topic = {
            if task.topic_source == "ai" {
                log::info!("[scheduler] 使用AI生成话题");
                
                let (api_key, base_url, model) = {
                    let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
                    
                    let api_key = match db_lock.get_ai_api_key().map_err(|e| format!("获取 API Key 失败: {}", e))? {
                        Some(key) => key.api_key,
                        None => return Err("AI API Key 未配置".to_string()),
                    };
                    
                    let config = db_lock.get_ai_config().map_err(|e| format!("获取 AI 配置失败: {}", e))?;
                    let base_url = config.as_ref().map(|c| c.base_url.clone()).unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
                    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_else(|| "deepseek-chat".to_string());
                    
                    (api_key, base_url, model)
                };
                
                let client = crate::ai::AIClient::new(&api_key, &base_url, &model, 0.7, 1000);
                let prompt = format!("请为题材\"{}\"生成一个吸引人的文章话题，只需要返回话题本身，不要有其他说明文字。", task.topic_category_name);
                
                let generated_topic = client.generate_article(&prompt, 50, "").await
                    .map_err(|e| format!("AI生成话题失败: {}", e))?;
                
                generated_topic.trim().to_string()
            } else if task.topic_source == "hot" {
                let hot_topic = crate::hot::pick_hot_topic(&task.hot_source)
                    .await
                    .map_err(|e| format!("获取热点话题失败: {}", e))?;
                log::info!("[scheduler] 使用热点雷达选题: {}", hot_topic);
                hot_topic
            } else {
                if let Some(ref pack_id) = task.topic_pack_id {
                    let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
                    
                    let topics = db_lock.get_topics_by_pack(Some(pack_id))
                        .map_err(|e| format!("获取话题失败: {}", e))?;
                    
                    if topics.is_empty() {
                        return Err("话题包中没有可用话题".to_string());
                    }
                    
                    let available_topics: Vec<_> = topics.iter()
                        .filter(|t| t.status == "available")
                        .collect();
                    
                    let selected_topic = if available_topics.is_empty() {
                        log::info!("[scheduler] 没有可用话题，重置所有话题状态");
                        for topic in topics.iter() {
                            let mut reset_topic = topic.clone();
                            reset_topic.status = "available".to_string();
                            db_lock.update_topic(&reset_topic)
                                .map_err(|e| format!("重置话题状态失败: {}", e))?;
                        }
                        let refreshed_topics = db_lock.get_topics_by_pack(Some(pack_id))
                            .map_err(|e| format!("获取话题失败: {}", e))?;
                        refreshed_topics.first()
                            .ok_or_else(|| "话题包中没有话题".to_string())?
                            .clone()
                    } else {
                        available_topics[0].clone()
                    };
                    let topic_content = selected_topic.content.clone();
                    
                    let mut used_topic = selected_topic;
                    used_topic.status = "used".to_string();
                    db_lock.update_topic(&used_topic)
                        .map_err(|e| format!("更新话题状态失败: {}", e))?;
                    
                    topic_content
                } else {
                    return Err("话题来源为话题包时，必须指定话题包ID".to_string());
                }
            }
        };
        
        log::info!("[scheduler] 选定话题: {}", topic);
        
        // 2-7. 获取博主、生成文章、保存文章（与之前相同）
        let blogger = {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.get_blogger(&task.blogger_id)
                .map_err(|e| format!("获取博主信息失败: {}", e))?
                .ok_or_else(|| "博主不存在".to_string())?
        };
        
        log::info!("[scheduler] 使用博主: {}", blogger.name);
        
        let methodology = crate::presets::methodology_prompt(&blogger, None);
        
        let (api_key, base_url, model, temperature, max_tokens) = {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            let api_key = match db_lock.get_ai_api_key().map_err(|e| format!("获取 API Key 失败: {}", e))? {
                Some(key) => key.api_key,
                None => return Err("AI API Key 未配置".to_string()),
            };
            let config = db_lock.get_ai_config().map_err(|e| format!("获取 AI 配置失败: {}", e))?;
            let base_url = config.as_ref().map(|c| c.base_url.clone()).unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
            let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_else(|| "deepseek-chat".to_string());
            let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
            let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
            (api_key, base_url, model, temperature, max_tokens)
        };
        
        log::info!("[scheduler] 开始生成文章...");
        let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
        let sys = format!("你是顶尖的公众号写手。输出要求：第一行是文章标题（不带书名号），随后空一行输出正文 Markdown。

{}", methodology);
        let usr = format!("请围绕话题《{}》写一篇约 {} 字的原创文章。", topic, task.word_count);
        let outcome = crate::quality::review_and_polish(&client, &sys, &usr, 2)
            .await
            .map_err(|e| format!("生成文章失败: {}", e))?;
        let content = outcome.content;
        log::info!("[scheduler] 文章生成成功，长度: {}，质量体检 {} 分（{} 轮）", content.len(), outcome.score, outcome.rounds);
        
        let title = client.generate_title(&content).await
            .map_err(|e| format!("生成标题失败: {}", e))?;
        let first_title = crate::ai::clean_title(&title, &topic);
        {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.record_quality_report(None, &first_title, "task", Some(&blogger.name), outcome.score, &outcome.report);
        }
        
        log::info!("[scheduler] 标题生成成功: {}", first_title);
        
        let article_id = format!("{}", chrono::Utc::now().timestamp_millis());
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        
        let article = crate::database::Article {
            id: article_id.clone(),
            title: first_title.clone(),
            content: content.clone(),
            cover_media_id: if task.cover_media_id.is_empty() { None } else { Some(task.cover_media_id.clone()) },
            author_name: task.author_name.clone(),
            category: Some(blogger.topic_category_name.clone()),
            tags: None,
            status: "draft".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            quality_report: Some(outcome.report.clone()),
        };
        
        {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.add_article(&article)
                .map_err(|e| format!("保存文章失败: {}", e))?;
        }
        
        log::info!("[scheduler] 文章已保存，ID: {}", article_id);
        
        // 8. 自动发布到微信公众号
        log::info!("[scheduler] 开始自动发布到微信公众号...");

        let saved_publish_status: String;

        let account = {
            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            
            let accounts = db_lock.get_all_accounts()
                .map_err(|e| format!("获取微信账户列表失败: {}", e))?;
            accounts.into_iter()
                .find(|a| a.id == task.account_id)
                .ok_or_else(|| "微信公众号账户不存在".to_string())?
        };
        
        let access_token = {
            let now_ts = chrono::Utc::now().timestamp();
            let account_key = account.id.clone();

            let need_refresh = {
                let cache = token_cache.lock().await;
                match cache.get(&account_key) {
                    Some((_token, expires_at)) => now_ts >= *expires_at,
                    None => true,
                }
            };

            if need_refresh {
                let wechat_client = crate::wechat::WechatClient::new(&account.app_id, &account.app_secret);
                let token_data = wechat_client.fetch_access_token().await
                    .map_err(|e| format!("获取 access_token 失败: {}", e))?;

                let mut cache = token_cache.lock().await;
                cache.insert(account_key, (token_data.access_token.clone(), token_data.expires_at));

                token_data.access_token
            } else {
                let cache = token_cache.lock().await;
                let (token, _expires_at) = cache.get(&account_key).unwrap();
                token.clone()
            }
        };
        
        let cover_media_id = if !task.cover_media_id.trim().is_empty() {
            Some(task.cover_media_id.as_str())
        } else if !account.default_cover_media_id.trim().is_empty() {
            Some(account.default_cover_media_id.as_str())
        } else {
            None
        };
        
        let wechat_client = crate::wechat::WechatClient::new(&account.app_id, &account.app_secret);
        
        let wechat_html = crate::ai::text_to_wechat_html(&content);
        let publish_result = wechat_client.create_draft(
            &access_token,
            &first_title,
            &wechat_html,
            task.author_name.as_deref(),
            cover_media_id,
        ).await
        .map_err(|e| format!("发布文章失败: {}", e))?;

        if publish_result.errcode != 0 {
            log::info!("[scheduler] 微信发布失败: {} (错误码: {})",
                       publish_result.errmsg, publish_result.errcode);
            saved_publish_status = "failed".to_string();

            let publish_history = crate::database::PublishHistory {
                id: format!("{}", chrono::Utc::now().timestamp_millis()),
                article_id: article_id.clone(),
                article_title: first_title.clone(),
                account_id: account.id.clone(),
                publish_time: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                status: "failed".to_string(),
            };

            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.add_publish_history(&publish_history)
                .map_err(|e| format!("记录发布历史失败: {}", e))?;
        } else {
            let is_mass = task.publish_type == "mass";
            saved_publish_status = if is_mass { "success".to_string() } else { "draft".to_string() };

            if is_mass {
                // 提交群发
                if let Some(ref media_id) = publish_result.media_id {
                    match wechat_client.publish_draft(&access_token, media_id).await {
                        Ok(submit_result) if submit_result.errcode == 0 => {
                            log::info!("[scheduler] 文章群发成功到微信公众号: {}", account.name);
                        }
                        Ok(submit_result) => {
                            log::warn!("[scheduler] 提交群发失败: {} (错误码: {})", submit_result.errmsg, submit_result.errcode);
                        }
                        Err(e) => {
                            log::warn!("[scheduler] 提交群发失败: {}", e);
                        }
                    }
                }
            } else {
                log::info!("[scheduler] 草稿已保存至微信公众号: {}", account.name);
            }

            let publish_status = if is_mass { "success" } else { "draft" };
            let publish_history = crate::database::PublishHistory {
                id: format!("{}", chrono::Utc::now().timestamp_millis()),
                article_id: article_id.clone(),
                article_title: first_title.clone(),
                account_id: account.id.clone(),
                publish_time: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                status: publish_status.to_string(),
            };

            let db_lock = db.lock().unwrap_or_else(|e| e.into_inner());
            db_lock.add_publish_history(&publish_history)
                .map_err(|e| format!("记录发布历史失败: {}", e))?;

            {
                let mut updated_article = article;
                updated_article.status = "published".to_string();
                db_lock.update_article(&updated_article)
                    .map_err(|e| format!("更新文章状态失败: {}", e))?;
                log::info!("[scheduler] 文章状态已更新为 published");
            }
        }
        
        log::info!("[scheduler] 定时任务执行完成");

        Ok((topic, first_title, saved_publish_status))
    }

    async fn update_next_execution_time(&self, task: &crate::database::ScheduledTask) -> Result<(), String> {
        let publish_time = match chrono::DateTime::parse_from_rfc3339(&task.publish_time) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                return Err(format!("任务 '{}' 的时间格式错误", task.name));
            }
        };

        let next_time = match task.schedule_type.as_str() {
            "daily" => publish_time + chrono::Duration::days(1),
            "weekly" => publish_time + chrono::Duration::weeks(1),
            "monthly" => {
                publish_time.checked_add_months(chrono::Months::new(1))
                    .ok_or_else(|| "无法计算下月执行时间".to_string())?
            }
            _ => return Ok(()),
        };

        let mut updated_task = task.clone();
        updated_task.publish_time = next_time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        updated_task.status = "pending".to_string();

        let db = self.db.lock().unwrap_or_else(|e| e.into_inner());
        db.update_task(&updated_task)
            .map_err(|e| format!("更新任务执行时间失败: {}", e))?;

        log::info!("[scheduler] 任务 '{}' 下次执行时间更新为: {}",
                  task.name, updated_task.publish_time);

        Ok(())
    }
}

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        // 仅在非运行时线程内尝试停止，避免 block_on panic
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        self.stop();
    }
}
