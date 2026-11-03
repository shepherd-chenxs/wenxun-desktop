//! 质量体检闭环：生成 → 五维打分 → 红线触发重写 → 取最高分版本。
//! 调度任务与手动生成（工坊/热点雷达）共用此路径，保证质量下限不依赖使用者水平。
//!
//! 评分采用分数锚点（rubric）制减少漂移；重写触发采用"红线制"而非总分线：
//! 任一维度 < 10（明确短板），或总分 < 70 且至少两个维度 < 12。

use crate::ai::AIClient;
use serde::Serialize;

/// 兼容旧语义的兜底总分线
pub const PASS_SCORE: i64 = 75;

pub const REVIEW_SYSTEM: &str = r#"你是一名苛刻的内容主编。请按以下 5 个维度给文章打分，每项 20 分。严格按每档描述对号入座，不要凭整体印象和稀泥。

【title 标题吸引力】
0-7：与正文脱节，或平铺直叙无信息量、无情绪钩子
8-14：有钩子但套路化（"在这个快节奏的时代…"式），或标题党而文不对题
15-20：具体、有张力、贴合选题，看到就想点开

【hook 开头钩子】
0-7：开头与主题脱节，或平铺直叙没有钩子
8-14：有钩子但套路化，前 3 句留不住着急的读者
15-20：具体、有张力、贴合本篇选题，前 3 句能明确感知到"钩住了"

【structure 结构节奏】
0-7：流水账、段落之间无推进感、前后重复
8-14：结构完整但平，段落推进靠惯性
15-20：分段有逻辑、层层递进，读起来有节奏感

【specificity 细节具体性】
0-7：全是正确的空话，没有一个具体案例/数字/场景
8-14：有具体内容但笼统，案例一带而过
15-20：案例、数字、场景具体可信，几乎找不到废话

【ai_flavor 人味】
0-7：AI 模板味浓重——工整排比、万能金句、机械的"首先/其次/总之"
8-14：偶有模板痕迹，整体尚可
15-20：像真人写的，有个人语气、偏见与温度

只输出 JSON（不要输出其他任何文字）：
{"title":18,"hook":15,"structure":17,"specificity":14,"ai_flavor":16,"total":80,"issues":["具体问题1，引用原文语句说明","问题2"]}

total 必须等于五项之和。issues 列出最影响质量的 2-4 个具体问题，引用原文语句佐证。"#;

const REWRITE_SYSTEM: &str = r#"你是一名资深改稿编辑。请基于原稿和主编指出的问题，重写出一篇全新的文章：只保留原稿的选题与可用素材，结构与表达全部重新组织，逐条修正问题。输出要求与原任务一致（若有标题要求，第一行仍是标题，不超过 50 个字）。"#;

#[derive(Debug, Clone, Serialize)]
pub struct QualityOutcome {
    pub content: String,
    pub report: String,
    pub score: i64,
    pub rounds: u32,
}

fn parse_score_json(text: &str) -> Result<serde_json::Value, String> {
    let cleaned = text.trim().trim_start_matches("```").trim_start_matches("json").trim_end_matches("```").trim();
    let start = cleaned.find('{').ok_or_else(|| format!("评分返回缺少 JSON: {}", &cleaned[..cleaned.len().min(100)]))?;
    let end = cleaned.rfind('}').ok_or_else(|| "评分返回缺少 JSON 结束符".to_string())?;
    serde_json::from_str(&cleaned[start..=end]).map_err(|e| format!("评分 JSON 解析失败: {}", e))
}

async fn score_article(client: &AIClient, article: &str) -> Result<serde_json::Value, String> {
    let raw = client
        .generate_content(REVIEW_SYSTEM, article, None, Some(0.2), Some(1000))
        .await?;
    let v = parse_score_json(&raw)?;
    if v["total"].is_null() {
        return Err("评分缺少 total 字段".to_string());
    }
    Ok(v)
}

/// 红线制：任一维度 < 10，或总分 < 70 且至少两个维度 < 12，才触发重写
fn needs_rewrite(score: &serde_json::Value) -> bool {
    let dim = |k: &str| score[k].as_i64().unwrap_or(20);
    let dims = [dim("title"), dim("hook"), dim("structure"), dim("specificity"), dim("ai_flavor")];
    let total = score["total"].as_i64().unwrap_or(100);
    dims.iter().any(|&d| d < 10) || (total < 70 && dims.iter().filter(|&&d| d < 12).count() >= 2)
}

fn report_markdown(score: &serde_json::Value, rounds: u32) -> String {
    let dim = |k: &str| score[k].as_i64().unwrap_or(0);
    let issues = score["issues"].as_array().cloned().unwrap_or_default();
    let mut md = format!(
        "| 标题 | 开头钩子 | 结构节奏 | 细节具体性 | 人味 | **总分** |\n|---|---|---|---|---|---|\n| {} | {} | {} | {} | {} | **{}** |",
        dim("title"), dim("hook"), dim("structure"), dim("specificity"), dim("ai_flavor"), dim("total")
    );
    if !issues.is_empty() {
        md.push_str("\n\n**遗留问题：**");
        for it in issues {
            if let Some(s) = it.as_str() {
                md.push_str(&format!("\n- {}", s));
            }
        }
    }
    if rounds > 0 {
        md.push_str(&format!("\n\n_经 {} 轮体检-重写后选定最优版本_", rounds));
    }
    md
}

/// 对已有文章（如用户改过的稿）重新体检，返回 (报告 markdown, 总分)
pub async fn recheck(client: &AIClient, content: &str) -> Result<(String, i64), String> {
    let score = score_article(client, content).await?;
    Ok((report_markdown(&score, 0), score["total"].as_i64().unwrap_or(0)))
}

/// 生成并体检：触发红线则带问题清单重写，最多 max_rewrites 轮，取最高分版本。
pub async fn review_and_polish(
    client: &AIClient,
    system_prompt: &str,
    user_prompt: &str,
    max_rewrites: u32,
) -> Result<QualityOutcome, String> {
    let mut article = client
        .generate_content(system_prompt, user_prompt, None, None, None)
        .await?;
    let mut best_score = score_article(client, &article).await?;
    let mut rounds: u32 = 0;

    while needs_rewrite(&best_score) && rounds < max_rewrites {
        rounds += 1;
        let issues = best_score["issues"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>()
            .join("\n- ");
        let rewrite_user = format!(
            "## 原始写作任务\n{}\n\n## 原稿\n{}\n\n## 主编指出的问题\n- {}\n\n请重写。只输出重写后的文章。",
            user_prompt, article, issues
        );
        let rewritten = client
            .generate_content(
                &format!("{}\n\n{}", system_prompt, REWRITE_SYSTEM),
                &rewrite_user,
                None,
                None,
                None,
            )
            .await?;
        let new_score = score_article(client, &rewritten).await?;
        article = rewritten;
        if new_score["total"].as_i64().unwrap_or(0) >= best_score["total"].as_i64().unwrap_or(0) {
            best_score = new_score;
        }
    }

    Ok(QualityOutcome {
        content: article,
        report: report_markdown(&best_score, rounds),
        score: best_score["total"].as_i64().unwrap_or(0),
        rounds,
    })
}
