// ============ 爆文工坊 ============

#[derive(serde::Serialize)]
pub struct WechatArticle {
    pub title: String,
    pub account: String,
    pub author: String,
    pub publish_time: String,
    pub cover: String,
    pub content_text: String,
}

/// 递归抽取块级文本：p/标题/引用/列表项成段，section/div 递归下钻，行内标签取纯文本
fn walk_block_text(
    el: scraper::ElementRef,
    out: &mut String,
) {
    for child in el.child_elements() {
        let tag = child.value().name();
        match tag {
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "li" => {
                let t: String = child.text().collect::<String>();
                let t = t.split_whitespace().collect::<Vec<&str>>().join(" ");
                if !t.is_empty() {
                    out.push_str(&t);
                    out.push_str("

");
                }
            }
            "section" | "div" | "ul" | "ol" => walk_block_text(child, out),
            _ => {
                let t: String = child.text().collect::<String>();
                let t = t.trim();
                if !t.is_empty() {
                    out.push_str(t);
                    out.push(' ');
                }
            }
        }
    }
}

/// 解析微信文章页 HTML（公开页面，无需登录）
fn parse_wechat_html(html: &str) -> Result<WechatArticle, String> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(&html);
    let text_of = |sel: &str| -> String {
        doc.select(&Selector::parse(sel).unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let meta_of = |prop: &str| -> String {
        Selector::parse(&format!("meta[property=\"{}\"]", prop)).ok()
            .and_then(|sel| doc.select(&sel).next())
            .and_then(|el| el.value().attr("content"))
            .unwrap_or("")
            .to_string()
    };

    // 环境验证页/失效页检测：抓不到标题和正文时给出明确提示
    let mut title = text_of("h1#activity-name");
    if title.is_empty() {
        title = meta_of("og:title");
    }


    let mut content_text = String::new();
    if let Some(el) = doc.select(&Selector::parse("#js_content").unwrap()).next() {
        walk_block_text(el, &mut content_text);
    }
    if content_text.trim().is_empty() {
        // 部分文章内容在分页容器/被折叠，退化为整页文本（截断到 2 万字符）
        let all: String = doc.select(&Selector::parse("#js_content, .rich_media_content").unwrap())
            .map(|el| el.text().collect::<String>())
            .collect::<String>();
        content_text = all.split_whitespace().collect::<Vec<&str>>().join(" ");
    }

    if title.is_empty() && content_text.trim().is_empty() {
        return Err("抓取失败：页面可能是环境验证页或链接已失效，请先在浏览器里确认能打开该文章".to_string());
    }

    let account = {
        let a = text_of("span#js_name, strong#js_name");
        if a.is_empty() { meta_of("og:site_name") } else { a }
    };
    let author = text_of("#js_author");
    let publish_time = text_of("em#publish_time");

    Ok(WechatArticle {
        title: if title.is_empty() { "未获取到标题".to_string() } else { title },
        account,
        author,
        publish_time,
        cover: meta_of("og:image"),
        content_text: content_text.trim().to_string(),
    })
}

/// 抓取微信公众号文章（无需登录：文章页是公开的）
#[tauri::command]
pub async fn fetch_wechat_article(url: String) -> Result<WechatArticle, String> {
    if !url.contains("mp.weixin.qq.com") {
        return Err("请输入微信公众平台文章链接（mp.weixin.qq.com）".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .no_proxy()
        .build()
        .map_err(|e| format!("创建请求失败: {}", e))?;

    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("文章请求失败: {}（请检查网络后重试）", e))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    parse_wechat_html(&html)
}

#[cfg(test)]
mod hot_workshop_tests {
    use super::*;

    const SAMPLE: &str = r#"<html><head>
        <meta property="og:title" content="备用标题">
        <meta property="og:image" content="https://mmbiz.qpic.cn/cover.jpg">
        </head><body>
        <h1 class="rich_media_title" id="activity-name">10 万+爆款标题在这里</h1>
        <span class="rich_media_meta rich_media_meta_nickname" id="js_name">测试公众号</span>
        <em id="publish_time">2026年9月5日</em>
        <div id="js_content">
            <section>
                <span>开头钩子文本。</span>
                <p>第一段完整内容，讲清楚一个观点。</p>
                <p>第二段内容，制造情绪共鸣。</p>
                <blockquote>金句引用</blockquote>
            </section>
        </div>
        </body></html>"#;

    #[test]
    fn parse_extracts_title_account_time_content() {
        let art = parse_wechat_html(SAMPLE).unwrap();
        assert_eq!(art.title, "10 万+爆款标题在这里");
        assert_eq!(art.account, "测试公众号");
        assert_eq!(art.publish_time, "2026年9月5日");
        assert_eq!(art.cover, "https://mmbiz.qpic.cn/cover.jpg");
        assert!(art.content_text.contains("第一段完整内容"));
        assert!(art.content_text.contains("金句引用"));
        assert!(art.content_text.contains("开头钩子文本"));
        assert!(art.content_text.contains("

"), "段与段之间应有换行");
    }

    #[test]
    fn parse_rejects_verification_page() {
        let r = parse_wechat_html("<html><head></head><body>环境异常</body></html>");
        assert!(r.is_err());
    }
}

