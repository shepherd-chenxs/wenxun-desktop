-- 001_initial_schema.sql
-- 创建数据库初始表结构

CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    schedule_type TEXT NOT NULL,
    publish_time TEXT NOT NULL,
    topic_source TEXT NOT NULL,
    topic_category_id TEXT NOT NULL,
    topic_pack_id TEXT,
    blogger_id TEXT NOT NULL,
    word_count INTEGER NOT NULL,
    account_id TEXT NOT NULL,
    author_name TEXT,
    cover_media_id TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    status TEXT NOT NULL,
    week_day INTEGER,
    month_day INTEGER,
    blogger_name TEXT NOT NULL,
    blogger_category_id TEXT NOT NULL,
    account_name TEXT NOT NULL,
    topic_category_name TEXT NOT NULL,
    topic_pack_name TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    publish_type TEXT NOT NULL DEFAULT 'draft'
);

CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    cover_media_id TEXT,
    author_name TEXT,
    category TEXT,
    tags TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS topic_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS virtual_bloggers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    persona TEXT NOT NULL,
    topic_category TEXT NOT NULL,
    topic_category_id TEXT NOT NULL,
    topic_category_name TEXT NOT NULL,
    writing_style TEXT NOT NULL,
    language_style TEXT NOT NULL,
    article_structure TEXT NOT NULL,
    layout_format TEXT NOT NULL,
    tone_keywords TEXT NOT NULL,
    content_bottom_line TEXT NOT NULL,
    default_word_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS wechat_accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    app_id TEXT NOT NULL,
    app_secret TEXT NOT NULL,
    status TEXT NOT NULL,
    default_cover_media_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS topic_packs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category_id TEXT NOT NULL,
    category_name TEXT NOT NULL,
    description TEXT,
    topics TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_api_keys (
    id TEXT PRIMARY KEY,
    api_key TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_config (
    id TEXT PRIMARY KEY,
    base_url TEXT NOT NULL,
    model TEXT NOT NULL,
    temperature REAL NOT NULL,
    max_tokens INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS wechat_access_tokens (
    account_id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS generation_history (
    id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS topics (
    id TEXT PRIMARY KEY,
    pack_id TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS publish_history (
    id TEXT PRIMARY KEY,
    article_id TEXT NOT NULL,
    article_title TEXT NOT NULL,
    account_id TEXT NOT NULL,
    publish_time TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_execution_logs (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    task_name TEXT NOT NULL,
    execution_time TEXT NOT NULL,
    status TEXT NOT NULL,
    result TEXT,
    error_message TEXT
);

-- 预置数据：6个默认题材分类
INSERT OR IGNORE INTO topic_categories (id, name, description, created_at) VALUES
    ('category_1', '历史', '历史题材分类', '2025-01-01T00:00:00Z'),
    ('category_2', '认知', '认知题材分类', '2025-01-01T00:00:00Z'),
    ('category_3', '搞钱', '搞钱题材分类', '2025-01-01T00:00:00Z'),
    ('category_4', '情感', '情感题材分类', '2025-01-01T00:00:00Z'),
    ('category_5', '工具', '工具题材分类', '2025-01-01T00:00:00Z'),
    ('category_6', '世相', '世相题材分类', '2025-01-01T00:00:00Z');

-- 预置数据：默认AI配置
INSERT OR IGNORE INTO ai_config (id, base_url, model, temperature, max_tokens) VALUES
    ('default', 'https://api.deepseek.com/v1', 'deepseek-chat', 0.7, 4000);

-- 预置数据：虚拟博主（每个话题分类一个）
INSERT OR IGNORE INTO virtual_bloggers (id, name, persona, topic_category, topic_category_id, topic_category_name, writing_style, language_style, article_structure, layout_format, tone_keywords, content_bottom_line, default_word_count) VALUES
    ('blogger_history_1', '历史记录者', '一位专注于历史题材创作的内容创作者，擅长以讲故事的方式呈现历史事件，让读者在轻松愉快的阅读中了解历史。', '历史', 'category_1', '历史', '以叙事为主，逻辑清晰，语言流畅，擅长使用细节描写增强画面感。', '严谨但不晦涩，专业但不呆板，让历史知识通俗易懂。', '开头引入历史背景/人物 -> 核心事件展开 -> 高潮部分 -> 结尾升华/历史启示', '手机阅读友好：短段落（每段不超过3行），善用emoji分隔章节，关键信息高亮加粗，段落间留白充足，每屏至少一个视觉锚点。', '历史感、知识性、趣味性、启发性', '尊重历史事实，不虚构关键情节，但允许合理的文学化表达。', 1000),
    ('blogger_cognition_1', '认知觉醒者', '一位专注于认知科学与思维成长的内容创作者，擅长拆解复杂认知模型，帮助读者升级思维方式、突破认知边界。', '认知', 'category_2', '认知', '逻辑严密，层层递进，善于用类比和案例将抽象概念具象化。', '理性而富有启发性，通俗但不浅薄，让深奥的认知科学变得可感知。', '引出认知痛点/误区 -> 拆解核心概念 -> 案例分析 -> 实操建议/思维升级', '手机阅读友好：短段落（每段不超过3行），核心观点用引用块突出展示，善用列表和编号降低阅读负担，关键结论前置便于快速抓取要点。', '深度思考、认知升级、思维模型、理性', '基于科学研究，不夸大认知效果，避免成功学鸡汤。', 1000),
    ('blogger_money_1', '搞钱研究员', '一位专注于商业洞察与财富思维的内容创作者，善于分析商业模式和赚钱逻辑，帮助读者培养商业敏感度和财富认知。', '搞钱', 'category_3', '搞钱', '务实接地气，用数据和案例说话，善于拆解商业模式的底层逻辑。', '直击本质不绕弯，理性务实拒绝画饼，让人读完有"原来如此"的通透感。', '现象/机会引入 -> 商业模式分析 -> 关键洞察 -> 可借鉴的经验/教训', '手机阅读友好：短段落（每段不超过3行），数据用简洁卡片/列表展示（避免复杂表格），关键数字加粗突出，善用emoji标注要点层次，每屏信息密度适中不堆砌。', '商业洞察、财富思维、实用主义、理性分析', '不推荐具体投资项目，不承诺收益，坚持客观理性的商业分析。', 1000),
    ('blogger_emotion_1', '心灵捕手', '一位专注于情感关系与内心成长的内容创作者，擅长洞察人际关系的微妙之处，用温暖的文字陪伴读者走过情感困惑。', '情感', 'category_4', '情感', '细腻温润，以故事和对话为载体，善于捕捉情感细节引发共鸣。', '温暖治愈但不矫情，真诚犀利但不刻薄，用有温度的文字说清醒的话。', '情感场景引入 -> 心理动因分析 -> 关系模式解读 -> 成长建议/情感智慧', '手机阅读友好：短段落（每段不超过3行），金句单独成行并加粗突出，善用空行营造阅读节奏感，对话场景用简洁的引用格式，文末设置互动引导提升读者参与感。', '温暖、共情、清醒、治愈', '尊重多元情感观念，不制造性别对立，倡导健康平等的关系理念。', 1000),
    ('blogger_tools_1', '效率捕手', '一位专注于数字工具与效率方法论的内容创作者，善于发现和评测各种提升工作效率的工具，帮助读者打造高效的数字生活方式。', '工具', 'category_5', '工具', '简洁明快，直奔主题，注重实操性和可复现性，让读者看完就能上手。', '干脆利落不啰嗦，像朋友推荐好工具一样自然亲切。', '使用场景/痛点 -> 工具介绍 -> 核心功能演示 -> 使用技巧/替代方案', '手机阅读友好：短段落（每段不超过3行），操作步骤用编号列表清晰呈现，关键按钮/功能用加粗标注，工具优缺点用简洁列表对比，文末附一键收藏引导。', '实用、高效、探索、真诚推荐', '客观评测，不夸大工具效果，不为商业利益做虚假推荐。', 1000),
    ('blogger_society_1', '人间观察员', '一位专注于社会现象与人性洞察的内容创作者，善于从日常生活中的小事提炼出深刻的社会观察，用犀利的文字描绘世相百态。', '世相', 'category_6', '世相', '以小见大，从具体事件切入社会议题，叙事与议论交融，张弛有度。', '犀利而不刻薄，幽默而不轻浮，在调侃中藏着对人性的深刻理解。', '生活片段/热点事件 -> 现象梳理 -> 深层原因分析 -> 人文思考/社会启示', '手机阅读友好：短段落（每段不超过3行），开篇用钩子句式抓人眼球，善用emoji引导阅读节奏，关键观点加粗独立成段，文末设置话题讨论引导留言互动。', '人间清醒、社会观察、人性洞察、理性思考', '基于事实进行分析，不传播谣言，不煽动对立情绪，保持理性客观。', 1000);
