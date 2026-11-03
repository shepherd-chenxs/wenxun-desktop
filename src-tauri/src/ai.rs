use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AICompletionRequest {
    pub model: String,
    pub messages: Vec<AIMessage>,
    pub temperature: f32,
    pub max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AICompletionResponse {
    pub id: String,
    pub choices: Vec<AIChoice>,
    pub usage: AIUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIChoice {
    pub message: AIMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

pub struct AIClient {
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: i32,
}

impl AIClient {
    pub fn new(api_key: &str, base_url: &str, model: &str, temperature: f32, max_tokens: i32) -> Self {
        AIClient {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
        }
    }

    pub async fn generate_content(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<i32>,
    ) -> Result<String, String> {
        let model = model.unwrap_or(&self.model);
        let temperature = temperature.unwrap_or(self.temperature);
        let max_tokens = max_tokens.unwrap_or(self.max_tokens);

        let request = AICompletionRequest {
            model: model.to_string(),
            messages: vec![
                AIMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                AIMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature,
            max_tokens,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .no_proxy()
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API request failed: {} - {}", status, text));
        }

        let result: AICompletionResponse = response
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(choice) = result.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No completion choices returned".to_string())
        }
    }

    pub async fn generate_article(
        &self,
        topic: &str,
        word_count: i32,
        style: &str,
    ) -> Result<String, String> {
        let system_prompt = "你是一个专业的文章写作助手，擅长创作各种类型的文章。你的任务是根据给定的主题创作高质量的文章。";

        let user_prompt = format!(
            "请根据以下主题写一篇约{}字的文章。要求：使用3到5个小标题分段。风格要求：{}\n\n主题：{}",
            word_count, style, topic
        );

        self.generate_content(system_prompt, &user_prompt, None, Some(0.7), Some(word_count * 2)).await
    }

    pub async fn generate_title(&self, content: &str) -> Result<String, String> {
        let system_prompt = "你是一个专业的文章标题创作助手。只返回一个标题，不要任何前缀、解释或额外文字。";
        let user_prompt = format!(
            "为以下文章生成一个吸引人的标题（不超过50字），直接返回标题本身：\n\n{}",
            &content.chars().take(1500).collect::<String>()
        );

        self.generate_content(system_prompt, &user_prompt, None, Some(0.8), Some(100)).await
    }

    pub async fn generate_summary(&self, content: &str) -> Result<String, String> {
        let system_prompt = "你是一个专业的文章摘要创作助手，擅长创作简洁有力的文章摘要。";
        let user_prompt = format!("请为以下文章内容生成一个简洁的摘要，不超过200个字：\n\n{}", content);

        self.generate_content(system_prompt, &user_prompt, None, Some(0.7), Some(500)).await
    }
}

/// HTML 字符转义
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 将 AI 生成的 Markdown/纯文本转换为适合微信公众号的 HTML
pub fn text_to_wechat_html(text: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;

    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() { continue; }

        let first_line = trimmed.lines().next().unwrap_or("");

        // ### 三级标题
        if first_line.starts_with("### ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            let heading = html_escape(first_line.trim_start_matches("### ").trim());
            html.push_str(&format!(
                "<h3 style=\"font-size: 20px; font-weight: bold; margin-bottom: 14px; color: #333;\">{}</h3>",
                heading
            ));
            continue;
        }

        // ## 二级标题
        if first_line.starts_with("## ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            let heading = html_escape(first_line.trim_start_matches("## ").trim());
            html.push_str(&format!(
                "<h2 style=\"font-size: 24px; font-weight: bold; margin-bottom: 16px; color: #333;\">{}</h2>",
                heading
            ));
            continue;
        }

        // # 一级标题
        if first_line.starts_with("# ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            let heading = html_escape(first_line.trim_start_matches("# ").trim());
            html.push_str(&format!(
                "<h1 style=\"font-size: 28px; font-weight: bold; margin-bottom: 20px; color: #333;\">{}</h1>",
                heading
            ));
            continue;
        }

        // 列表项 (每行都以 - 或 * 开头)
        let is_list_paragraph = trimmed.lines().all(|l| {
            let t = l.trim();
            t.starts_with("- ") || t.starts_with("* ")
        });

        if is_list_paragraph {
            if !in_list {
                html.push_str("<ul style=\"margin-bottom: 16px; padding-left: 2em;\">");
                in_list = true;
            }
            for line in trimmed.lines() {
                let item = html_escape(line.trim().trim_start_matches("- ").trim_start_matches("* "));
                html.push_str(&format!(
                    "<li style=\"margin-bottom: 8px; line-height: 1.6;\">{}</li>",
                    item
                ));
            }
            continue;
        }

        // 普通段落
        if in_list { html.push_str("</ul>"); in_list = false; }
        html.push_str("<p style=\"margin-bottom: 16px; line-height: 1.8; text-indent: 2em;\">");
        for (i, line) in trimmed.lines().enumerate() {
            if i > 0 { html.push_str("<br>"); }
            // 先 HTML 转义，再处理行内粗体 **text**（确保 <strong> 不被转义）
            html.push_str(&inline_format(&html_escape(line)));
        }
        html.push_str("</p>");
    }

    if in_list { html.push_str("</ul>"); }
    html
}

/// 处理行内格式：粗体 **text**
fn inline_format(line: &str) -> String {
    let mut result = String::new();
    let mut remaining = line;
    while let Some(start) = remaining.find("**") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("**") {
            result.push_str(&format!("<strong>{}</strong>", &after_start[..end]));
            remaining = &after_start[end + 2..];
        } else {
            result.push_str("**");
            remaining = after_start;
        }
    }
    result.push_str(remaining);
    result
}

/// 清理 AI 返回的标题，去除废话前缀，提取真正的标题
pub fn clean_title(raw: &str, fallback: &str) -> String {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        // 跳过明显的废话前缀行
        if trimmed.starts_with("好的") || trimmed.starts_with("以下是") || trimmed.starts_with("为您")
            || trimmed.starts_with("当然") || trimmed.starts_with("OK") || trimmed.starts_with("ok")
            || trimmed.starts_with("这里") || trimmed.starts_with("我") || trimmed.starts_with("让我")
            || trimmed.starts_with("根据") || trimmed.starts_with("基于") || trimmed.starts_with("已为您")
            || trimmed.starts_with("精心") {
            continue;
        }
        // 去掉编号前缀 "1." "1、" "#1" "标题1：" 等，去掉书名号
        let stripped = trimmed
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '、' || c == '#' || c == ' ')
            .trim_start_matches(|c: char| c == '：' || c == ':' || c == ' ')
            .trim_start_matches("标题")
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '、')
            .trim()
            .trim_matches('《')
            .trim_matches('》');
        if stripped.len() >= 5 && stripped.len() <= 60 {
            return stripped.to_string();
        }
    }
    // 兜底：用第一行非空内容
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.len() <= 60 {
            let stripped = trimmed
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '、' || c == '#' || c == ' ')
                .trim_start_matches(|c: char| c == '：' || c == ':' || c == ' ')
                .trim_start_matches("标题")
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '、')
                .trim()
                .trim_matches('《')
                .trim_matches('》');
            if stripped.len() >= 3 {
                return stripped.to_string();
            }
        }
    }
    fallback.to_string()
}
