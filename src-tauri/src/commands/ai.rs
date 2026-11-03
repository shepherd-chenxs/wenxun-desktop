use crate::database::{AIApiKey, AIConfig};
use crate::state::AppState;
use tauri::State;

// ============ AI API Keys ============
#[tauri::command]
pub fn get_ai_api_key(state: State<AppState>) -> Result<Option<AIApiKey>, String> {
    let db = lock_db!(state);
    db.get_ai_api_key()
        .map_err(|e| format!("获取 API Key 失败: {}", e))
}

#[tauri::command]
pub fn save_ai_api_key(state: State<AppState>, api_key: AIApiKey) -> Result<bool, String> {
    let db = lock_db!(state);
    db.save_ai_api_key(&api_key)
        .map_err(|e| format!("Failed to save API key: {}", e))
        .and(Ok(true))
}

#[tauri::command]
pub fn get_ai_config(state: State<AppState>) -> Result<Option<AIConfig>, String> {
    let db = lock_db!(state);
    db.get_ai_config()
        .map_err(|e| format!("获取 AI 配置失败: {}", e))
}

#[tauri::command]
pub fn save_ai_config(state: State<AppState>, config: AIConfig) -> Result<bool, String> {
    let db = lock_db!(state);
    db.save_ai_config(&config)
        .map_err(|e| format!("Failed to save AI config: {}", e))
        .and(Ok(true))
}

// ============ AI Generation ============
#[tauri::command]
pub async fn generate_article(
    state: State<'_, AppState>,
    topic: String,
    word_count: i32,
    style: String,
) -> Result<String, String> {
    let (api_key, base_url, model, temperature, max_tokens) = {
        let db = lock_db!(state);
        let api_key = match db.get_ai_api_key().unwrap_or(None) {
            Some(key) => key.api_key,
            None => return Err("AI API key not configured".to_string()),
        };
        let config = db.get_ai_config().unwrap_or(None);
        let base_url = config
            .as_ref()
            .map(|c| c.base_url.clone())
            .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
        let model = config
            .as_ref()
            .map(|c| c.model.clone())
            .unwrap_or_else(|| "deepseek-chat".to_string());
        let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
        let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
        (api_key, base_url, model, temperature, max_tokens)
    };

    let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
    client.generate_article(&topic, word_count, &style).await
}

#[tauri::command]
pub async fn generate_title(state: State<'_, AppState>, content: String) -> Result<String, String> {
    let (api_key, base_url, model, temperature, max_tokens) = {
        let db = lock_db!(state);
        let api_key = match db.get_ai_api_key().unwrap_or(None) {
            Some(key) => key.api_key,
            None => return Err("AI API key not configured".to_string()),
        };
        let config = db.get_ai_config().unwrap_or(None);
        let base_url = config
            .as_ref()
            .map(|c| c.base_url.clone())
            .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
        let model = config
            .as_ref()
            .map(|c| c.model.clone())
            .unwrap_or_else(|| "deepseek-chat".to_string());
        let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
        let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
        (api_key, base_url, model, temperature, max_tokens)
    };

    let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
    client.generate_title(&content).await
}

#[tauri::command]
pub async fn generate_summary(
    state: State<'_, AppState>,
    content: String,
) -> Result<String, String> {
    let (api_key, base_url, model, temperature, max_tokens) = {
        let db = lock_db!(state);
        let api_key = match db.get_ai_api_key().unwrap_or(None) {
            Some(key) => key.api_key,
            None => return Err("AI API key not configured".to_string()),
        };
        let config = db.get_ai_config().unwrap_or(None);
        let base_url = config
            .as_ref()
            .map(|c| c.base_url.clone())
            .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
        let model = config
            .as_ref()
            .map(|c| c.model.clone())
            .unwrap_or_else(|| "deepseek-chat".to_string());
        let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
        let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
        (api_key, base_url, model, temperature, max_tokens)
    };

    let client = crate::ai::AIClient::new(&api_key, &base_url, &model, temperature, max_tokens);
    client.generate_summary(&content).await
}

/// 对已有文章（通常是用户改过的稿）重新体检，报告写回文章
#[tauri::command]
pub async fn recheck_quality(
    state: State<'_, AppState>,
    article_id: String,
    content: String,
) -> Result<(String, i64), String> {
    let client = ai_client(&state)?;
    let (report, score) = crate::quality::recheck(&client, &content).await?;
    {
        let db = lock_db!(state);
        let mut article = db
            .get_article(&article_id)
            .map_err(|e| format!("读取文章失败: {}", e))?
            .ok_or_else(|| "文章不存在".to_string())?;
        article.quality_report = Some(report.clone());
        db.update_article(&article)
            .map_err(|e| format!("保存报告失败: {}", e))?;
        db.record_quality_report(
            Some(&article_id),
            &article.title,
            "recheck",
            None,
            score,
            &report,
        );
    }
    Ok((report, score))
}

/// 按体检报告对当前稿做 AI 修订，返回修订稿（是否采纳由用户决定，前端对比展示）
#[tauri::command]
pub async fn revise_with_report(
    state: State<'_, AppState>,
    content: String,
    report: String,
) -> Result<String, String> {
    let client = ai_client(&state)?;
    let system = "你是一名资深改稿编辑。请严格依据质量体检报告指出的问题修订文章：只改报告点名的部分与直接相关的表达，保留原稿的结构、语气与已达标的部分。输出要求与原稿一致（若原稿第一行是标题，保持第一行为标题，不超过 50 个字）。只输出修订后的全文。";
    let user = format!(
        "## 原稿\n{}\n\n## 质量体检报告\n{}\n\n请输出修订后的全文。",
        content, report
    );
    client
        .generate_content(system, &user, None, None, None)
        .await
}

#[derive(Debug, serde::Serialize)]
pub struct PersonaImprovement {
    pub summary: String,
    pub suggestions: Vec<String>,
    pub updates: std::collections::HashMap<String, String>,
    pub report_count: i64,
    /// 勾选报告对应的写手是否为自定义写手（只有自定义写手才允许应用修订）
    pub can_apply: bool,
    /// 自定义写手 ID，可应用时返回
    pub blogger_id: Option<String>,
    /// 写手名（内置/自定义/无则为空，空表示【当前用户】）
    pub blogger_name: Option<String>,
}

/// 综合分析：按用户勾选的质检报告聚类分析，输出弱项模式、提升建议和（自定义写手）prompt 修订稿。
/// 是否采纳由用户决定，逐字段对比应用。
#[tauri::command]
pub async fn analyze_selected_quality_reports(
    state: State<'_, AppState>,
    report_ids: Vec<String>,
) -> Result<PersonaImprovement, String> {
    if report_ids.is_empty() {
        return Err("请先勾选要分析的质检报告".to_string());
    }

    let (all_reports, all_bloggers) = {
        let db = lock_db!(state);
        let all_reports = db
            .get_quality_reports(None, 10000)
            .map_err(|e| e.to_string())?;
        let all_bloggers = db.get_all_bloggers().map_err(|e| e.to_string())?;
        (all_reports, all_bloggers)
    };

    let wanted: std::collections::HashSet<String> = report_ids.into_iter().collect();
    let reports: Vec<crate::database::QualityReportRow> = all_reports
        .into_iter()
        .filter(|r| wanted.contains(&r.id))
        .collect();
    if reports.is_empty() {
        return Err("勾选的质检报告不存在或已被删除".to_string());
    }

    let mut persona_names: Vec<String> = reports
        .iter()
        .filter_map(|r| r.persona_name.clone().filter(|n| !n.trim().is_empty()))
        .collect();
    persona_names.sort();
    persona_names.dedup();
    if persona_names.len() > 1 {
        return Err(
            "勾选的报告来自多个写手，请先筛选到同一写手（或同一批【当前用户】记录）后再生成建议"
                .to_string(),
        );
    }
    let persona_name = persona_names.first().cloned();

    // 优先匹配自定义写手；同名内置写手仅作为兜底，仍不允许应用
    let blogger = persona_name
        .as_ref()
        .and_then(|name| {
            all_bloggers
                .iter()
                .find(|b| b.name == *name && !b.is_builtin)
                .cloned()
        })
        .or_else(|| {
            persona_name
                .as_ref()
                .and_then(|name| all_bloggers.iter().find(|b| b.name == *name).cloned())
        });
    let can_apply = blogger.as_ref().map(|b| !b.is_builtin).unwrap_or(false);

    let mut digest = String::new();
    for r in &reports {
        digest.push_str(&format!(
            "### 报告（{} 分，{}，写手：{}）
文章：{}
{}

",
            r.score,
            r.created_at,
            r.persona_name
                .clone()
                .unwrap_or_else(|| "【当前用户】".to_string()),
            r.article_title,
            r.report
        ));
    }
    if digest.len() > 40_000 {
        digest.truncate(40_000);
    }

    let system = if can_apply {
        r#"你是一名内容策略教练。以下是一位写手当前的方法论配置，以及多份其产出的文章质量体检报告。请：
1. 从报告的"遗留问题"中聚类出反复出现的弱项模式（注明出现次数）；
2. 给出 3-5 条具体可执行的提升建议；
3. 针对弱项，给出该人设方法论字段的修订稿。

只输出 JSON（不要其他文字）：
{"summary":"弱项模式总结（150字内，注明每个模式的出现次数）","suggestions":["建议1","建议2"],"updates":{"writing_style":"修订后的完整字段文本"}}

updates 只包含需要修改的字段（可选项：writing_style/language_style/article_structure/layout_format/tone_keywords/content_bottom_line），值为该字段修订后的完整文本；不需要修改的字段不要出现在 updates 里。如果报告太少难以归因，在 summary 开头注明"样本有限："。"#
    } else {
        r#"你是一名内容策略教练。以下是若干份文章质量体检报告。请：
1. 从报告的"遗留问题"中聚类出反复出现的弱项模式（注明出现次数）；
2. 给出 3-5 条具体可执行的提升建议。

只输出 JSON（不要其他文字）：
{"summary":"弱项模式总结（150字内，注明每个模式的出现次数）","suggestions":["建议1","建议2"]}
如果报告太少难以归因，在 summary 开头注明"样本有限："。"#
    };

    let mut user = String::new();
    if let Some(b) = &blogger {
        user.push_str(&format!(
            "## 该写手当前方法论配置
写手名：{}
写手描述：{}
选题与表达要领：{}
语言风格：{}
文章结构：{}
排版规范：{}
语气关键词：{}
内容红线：{}

",
            b.name,
            b.persona,
            b.writing_style,
            b.language_style,
            b.article_structure,
            b.layout_format,
            b.tone_keywords,
            b.content_bottom_line
        ));
    }
    user.push_str(&format!(
        "## 质检报告（共 {} 份）
{}",
        reports.len(),
        digest
    ));

    let client = ai_client(&state)?;
    let raw = client
        .generate_content(system, &user, None, None, None)
        .await?;
    let cleaned = raw.trim();
    let start = cleaned
        .find('{')
        .ok_or_else(|| format!("分析返回缺少 JSON: {}", &cleaned[..cleaned.len().min(100)]))?;
    let end = cleaned
        .rfind('}')
        .ok_or_else(|| "分析返回缺少 JSON 结束符".to_string())?;
    let v: serde_json::Value = serde_json::from_str(&cleaned[start..=end])
        .map_err(|e| format!("分析 JSON 解析失败: {}", e))?;

    let get_str = |k: &str| v[k].as_str().unwrap_or("").trim().to_string();
    let suggestions = v["suggestions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    let mut updates = std::collections::HashMap::new();
    if let Some(obj) = v["updates"].as_object() {
        const ALLOWED: &[&str] = &[
            "writing_style",
            "language_style",
            "article_structure",
            "layout_format",
            "tone_keywords",
            "content_bottom_line",
        ];
        for (k, val) in obj {
            if ALLOWED.contains(&k.as_str()) {
                if let Some(s) = val.as_str() {
                    if !s.trim().is_empty() {
                        updates.insert(k.clone(), s.trim().to_string());
                    }
                }
            }
        }
    }
    if !can_apply {
        updates.clear();
    }

    Ok(PersonaImprovement {
        summary: if get_str("summary").is_empty() {
            "（AI 未返回总结）".to_string()
        } else {
            get_str("summary")
        },
        suggestions,
        updates,
        report_count: reports.len() as i64,
        can_apply,
        blogger_id: if can_apply {
            blogger.as_ref().map(|b| b.id.clone())
        } else {
            None
        },
        blogger_name: persona_name,
    })
}

/// 质检库报告列表
#[tauri::command]
pub fn get_quality_report_list(
    state: State<'_, AppState>,
    persona_name: Option<String>,
) -> Result<Vec<crate::database::QualityReportRow>, String> {
    let db = lock_db!(state);
    db.get_quality_reports(persona_name.as_deref(), 10000)
        .map_err(|e| format!("读取质检库失败: {}", e))
}

/// 删除质检库里的单条报告
#[tauri::command]
pub fn delete_quality_report(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    let db = lock_db!(state);
    db.delete_quality_report(&id)
        .map_err(|e| format!("删除质检报告失败: {}", e))?;
    Ok(true)
}

/// 把一条体检教训记入写作人设（每行一条，上限 5 条，超出挤掉最旧的）
#[tauri::command]
pub fn add_lesson_to_blogger(
    state: State<'_, AppState>,
    blogger_id: String,
    lesson: String,
) -> Result<(), String> {
    let lesson = lesson.trim().to_string();
    if lesson.is_empty() {
        return Err("教训内容为空".to_string());
    }
    let db = lock_db!(state);
    let mut blogger = db
        .get_blogger(&blogger_id)
        .map_err(|e| format!("读取人设失败: {}", e))?
        .ok_or_else(|| "人设不存在".to_string())?;
    let mut lines: Vec<String> = blogger
        .lessons
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.iter().any(|l| l == &lesson) {
        return Ok(());
    }
    lines.push(lesson);
    if lines.len() > 5 {
        lines = lines.split_off(lines.len() - 5);
    }
    blogger.lessons = lines.join("\n");
    db.update_blogger(&blogger)
        .map_err(|e| format!("保存失败: {}", e))
}

/// 通用 AI 对话（爆文工坊的拆解/写同款通过它实现，复用现有 AI 配置）
#[tauri::command]
pub async fn ai_chat(
    state: State<'_, AppState>,
    system_prompt: String,
    user_prompt: String,
) -> Result<String, String> {
    let client = ai_client(&state)?;
    client
        .generate_content(&system_prompt, &user_prompt, None, None, None)
        .await
}

fn ai_client(state: &State<'_, AppState>) -> Result<crate::ai::AIClient, String> {
    let db = lock_db!(state);
    let api_key = match db.get_ai_api_key().unwrap_or(None) {
        Some(key) => key.api_key,
        None => return Err("AI API key 未配置，请先到 AI 设置中配置".to_string()),
    };
    let config = db.get_ai_config().unwrap_or(None);
    let base_url = config
        .as_ref()
        .map(|c| c.base_url.clone())
        .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
    let model = config
        .as_ref()
        .map(|c| c.model.clone())
        .unwrap_or_else(|| "deepseek-chat".to_string());
    let temperature = config.as_ref().map(|c| c.temperature).unwrap_or(0.7);
    let max_tokens = config.as_ref().map(|c| c.max_tokens).unwrap_or(4000);
    Ok(crate::ai::AIClient::new(
        &api_key,
        &base_url,
        &model,
        temperature,
        max_tokens,
    ))
}

const WRITER_BASE: &str = "你是顶尖的公众号写手。输出要求：第一行是文章标题（不带书名号、不要\"标题：\"前缀，不超过 50 个字），随后空一行输出正文 Markdown。不要输出任何解释性文字。";

#[derive(Debug, serde::Serialize)]
pub struct QualityGateResult {
    pub title: String,
    pub content: String,
    pub report: String,
    pub score: i64,
    pub rounds: u32,
}

fn split_title(content: &str, fallback: &str) -> (String, String) {
    let mut lines = content.trim().splitn(2, '\n');
    let raw = lines.next().unwrap_or("").trim();
    let title = raw
        .strip_prefix("标题：")
        .or_else(|| raw.strip_prefix("标题:"))
        .unwrap_or(raw)
        .trim();
    let title = if title.is_empty() {
        fallback.to_string()
    } else {
        title.to_string()
    };
    let body = lines.next().unwrap_or("").trim().to_string();
    (title, body)
}

/// 手动生成的统一入口：自带质量体检闭环（打分→条件重写→报告）。
/// 工坊"写同款"走这里；knobs 为一次性微调旋钮。
#[tauri::command]
pub async fn generate_with_quality_gate(
    state: State<'_, AppState>,
    system_prompt: String,
    user_prompt: String,
    knobs: Option<crate::presets::Knobs>,
    persona_name: Option<String>,
) -> Result<QualityGateResult, String> {
    let client = ai_client(&state)?;
    let sys = match &knobs {
        Some(k) if !k.is_empty() => format!(
            "{}\n\n{}",
            system_prompt,
            crate::presets::knobs_suffix(Some(k))
        ),
        _ => system_prompt,
    };
    let outcome = crate::quality::review_and_polish(&client, &sys, &user_prompt, 2).await?;
    let (title, content) = split_title(&outcome.content, "生成文章");
    {
        let db = lock_db!(state);
        db.record_quality_report(
            None,
            &title,
            "workshop",
            persona_name.as_deref(),
            outcome.score,
            &outcome.report,
        );
    }
    Ok(QualityGateResult {
        title,
        content,
        report: outcome.report,
        score: outcome.score,
        rounds: outcome.rounds,
    })
}

/// 热点雷达一键成文：支持内置/自定义写手，按所选人设 + 热点话题生成，同样过质量体检。
#[tauri::command]
pub async fn generate_blogger_article(
    state: State<'_, AppState>,
    blogger_id: String,
    platform: Option<String>,
    topic: String,
    word_count: i32,
    knobs: Option<crate::presets::Knobs>,
) -> Result<QualityGateResult, String> {
    let blogger = {
        let db = lock_db!(state);
        db.get_blogger(&blogger_id)
            .map_err(|e| format!("读取写手失败: {}", e))?
            .ok_or_else(|| "所选写手不存在，请刷新后重试".to_string())?
    };
    let client = ai_client(&state)?;

    let sys = format!(
        "{}\n\n{}",
        WRITER_BASE,
        crate::presets::methodology_prompt(&blogger, knobs.as_ref())
    );
    let platform_note = match platform.as_deref() {
        Some(p) if !p.is_empty() => format!(
            "（话题来自{}热榜，写作时可利用其时效性，但不要直接提及榜单来源）",
            p
        ),
        _ => String::new(),
    };
    let user_prompt = format!(
        "请围绕话题《{}》{}写一篇约 {} 字的原创文章。",
        topic, platform_note, word_count
    );

    let outcome = crate::quality::review_and_polish(&client, &sys, &user_prompt, 2).await?;
    let (title, content) = split_title(&outcome.content, &topic);
    {
        let db = lock_db!(state);
        db.record_quality_report(
            None,
            &title,
            "radar",
            Some(blogger.name.as_str()),
            outcome.score,
            &outcome.report,
        );
    }
    Ok(QualityGateResult {
        title,
        content,
        report: outcome.report,
        score: outcome.score,
        rounds: outcome.rounds,
    })
}

/// 从爆文拆解报告沉淀一个自定义写作人设（AI 提取五维配置并入库）
#[tauri::command]
pub async fn create_blogger_from_analysis(
    state: State<'_, AppState>,
    analysis: String,
) -> Result<crate::database::VirtualBlogger, String> {
    let client = ai_client(&state)?;
    let raw = client
        .generate_content(
            crate::presets::PERSONA_EXTRACT_SYSTEM,
            &analysis,
            None,
            None,
            None,
        )
        .await?;
    let cleaned = raw.trim();
    let start = cleaned.find('{').ok_or_else(|| {
        format!(
            "人设提取返回缺少 JSON: {}",
            &cleaned[..cleaned.len().min(100)]
        )
    })?;
    let end = cleaned
        .rfind('}')
        .ok_or_else(|| "人设提取返回缺少 JSON 结束符".to_string())?;
    let v: serde_json::Value = serde_json::from_str(&cleaned[start..=end])
        .map_err(|e| format!("人设 JSON 解析失败: {}", e))?;

    let get = |k: &str| v[k].as_str().unwrap_or("").trim().to_string();
    let name = {
        let n = get("name");
        if n.is_empty() {
            "自定义人设".to_string()
        } else {
            n
        }
    };
    let blogger = crate::database::VirtualBlogger {
        id: format!("{}", chrono::Utc::now().timestamp_millis()),
        name,
        persona: get("persona"),
        topic_category: String::new(),
        topic_category_id: String::new(),
        topic_category_name: String::new(),
        writing_style: get("writing_style"),
        language_style: get("language_style"),
        article_structure: get("article_structure"),
        layout_format: get("layout_format"),
        tone_keywords: get("tone_keywords"),
        content_bottom_line: get("content_bottom_line"),
        default_word_count: 1500,
        is_builtin: false,
        preset_key: String::new(),
        lessons: String::new(),
    };
    {
        let db = lock_db!(state);
        db.add_blogger(&blogger)
            .map_err(|e| format!("保存人设失败: {}", e))?;
    }
    Ok(blogger)
}

/// 智能体检：AI 创作归属参考检测（仿朱雀的判定维度，但结果不代表朱雀等官方检测）
#[tauri::command]
pub async fn detect_ai_authorship(
    state: State<'_, AppState>,
    content: String,
    text_type: Option<String>,
) -> Result<crate::smart_check::AiFlavorReport, String> {
    let content = content.trim().to_string();
    let len = content.chars().count();
    if len < crate::smart_check::MIN_INPUT_CHARS {
        return Err(format!(
            "内容太短（至少 {} 字），特征不足难以有效检测",
            crate::smart_check::MIN_INPUT_CHARS
        ));
    }
    if len > crate::smart_check::MAX_INPUT_CHARS {
        return Err(format!(
            "内容过长（最多 {} 字），请分段检测",
            crate::smart_check::MAX_INPUT_CHARS
        ));
    }
    let text_type = match text_type.as_deref() {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => "自媒体文章（公众号等）".to_string(),
    };
    let client = ai_client(&state)?;
    crate::smart_check::detect_ai_flavor(&client, &content, &text_type).await
}

/// 智能体检：一键去 AI 味改写，返回改写后的完整文章
#[tauri::command]
pub async fn humanize_ai_text(
    state: State<'_, AppState>,
    content: String,
    style: Option<String>,
) -> Result<String, String> {
    let content = content.trim().to_string();
    let len = content.chars().count();
    if len < crate::smart_check::MIN_INPUT_CHARS {
        return Err(format!(
            "内容太短（至少 {} 字），请粘贴完整稿件",
            crate::smart_check::MIN_INPUT_CHARS
        ));
    }
    if len > crate::smart_check::MAX_INPUT_CHARS {
        return Err(format!(
            "内容过长（最多 {} 字），请分段处理",
            crate::smart_check::MAX_INPUT_CHARS
        ));
    }
    let style = match style.as_deref() {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => "公众号文章，自然、口语但有信息密度".to_string(),
    };
    let client = ai_client(&state)?;
    crate::smart_check::humanize_ai_flavor(&client, &content, &style).await
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeAiRecheckResult {
    /// 去 AI 味后的完整成稿
    pub text: String,
    /// 对成稿再次执行原创度检测的结果
    pub report: crate::smart_check::AiFlavorReport,
}

/// 智能体检闭环：一键去 AI 味 → 自动复检，返回成稿和复检报告。
#[tauri::command]
pub async fn humanize_ai_text_with_check(
    state: State<'_, AppState>,
    content: String,
    style: Option<String>,
    text_type: Option<String>,
) -> Result<DeAiRecheckResult, String> {
    let content = content.trim().to_string();
    let len = content.chars().count();
    if len < crate::smart_check::MIN_INPUT_CHARS {
        return Err(format!(
            "内容太短（至少 {} 字），请粘贴完整稿件",
            crate::smart_check::MIN_INPUT_CHARS
        ));
    }
    if len > crate::smart_check::MAX_INPUT_CHARS {
        return Err(format!(
            "内容过长（最多 {} 字），请分段处理",
            crate::smart_check::MAX_INPUT_CHARS
        ));
    }
    let style = match style.as_deref() {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => "公众号文章，自然、口语但有信息密度".to_string(),
    };
    let text_type = match text_type.as_deref() {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => "自媒体文章（公众号等）".to_string(),
    };
    let client = ai_client(&state)?;
    let text = crate::smart_check::humanize_ai_flavor(&client, &content, &style).await?;
    let report = crate::smart_check::detect_ai_flavor(&client, &text, &text_type).await?;
    Ok(DeAiRecheckResult { text, report })
}
