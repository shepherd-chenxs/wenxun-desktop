//! 本地 AI 特征规则扫描（零 token）。
//!
//! 说明：检测结果基于可解释的统计与中文“AI 腔”词表规则，是近似信号，并不等同于
//! 腾讯朱雀等任何官方检测器的结果；本项目不对外部检测工具作“保证通过”类承诺。
//!
//! 词表/句式正则/确定性改写映射的部分规则集移植自 MIT 开源项目
//! yotta-humanize（YottaMeta，https://github.com/YottaMeta/yotta-humanize），
//! Copyright (c) 2026 YottaMeta，按 MIT License 使用；完整许可见仓库 LICENSE。

use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

/// 句末/句中断点（中文句号、问号、叹号、分号及换行）
const SENTENCE_END: &[char] = &['。', '！', '？', '?', '!', '；', ';', '\n'];

/// 论文腔/公众号 AI 过渡连接词
const AI_CONNECTORS: &[&str] = &[
    "综上所述",
    "总而言之",
    "总的来说",
    "由此可见",
    "值得注意的是",
    "不难发现",
    "换句话说",
    "从这个角度看",
    "与此同时",
    "然而",
    "此外",
    "因此",
    "所以",
    "同时",
    "首先",
    "其次",
    "最后",
    "一方面",
    "另一方面",
    "不仅",
    "而且",
    "这意味着",
];

/// 自媒体语境里的 AI/营销高频词（出现不代表 AI，但密度过高是风险信号）
const AI_PHRASES: &[&str] = &[
    "赋能",
    "底层逻辑",
    "颗粒度",
    "抓手",
    "闭环",
    "破圈",
    "沉淀",
    "洞察",
    "红利",
    "赛道",
    "方法论",
    "降维打击",
    "信息茧房",
    "情绪价值",
    "在当今社会",
    "在这个快节奏的时代",
    "让我们一起",
    "我们需要明白",
    "需要注意的是",
    "不难看出",
    "显而易见",
    "这意味着",
    "综上所述",
    "总而言之",
    "值得注意的是",
];

/// MATTR 滑窗长度
const MATTR_WINDOW: usize = 50;
/// MATTR 步长（越大越快、精度略低）
const MATTR_STEP: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RuleGroup {
    Content,
    Language,
    Style,
    Communication,
    Filler,
}

impl RuleGroup {
    fn code(self) -> &'static str {
        match self {
            RuleGroup::Content => "content",
            RuleGroup::Language => "language",
            RuleGroup::Style => "style",
            RuleGroup::Communication => "communication",
            RuleGroup::Filler => "filler",
        }
    }

    fn label(self) -> &'static str {
        match self {
            RuleGroup::Content => "内容（空泛/套话/归因）",
            RuleGroup::Language => "语言（黑话/堆叠/强调）",
            RuleGroup::Style => "句式（排比/列举/标点）",
            RuleGroup::Communication => "沟通（残渣/奉承/回避）",
            RuleGroup::Filler => "废话（填充/转折/声明）",
        }
    }
}

/// 一类规则的静态定义（词表规则移植自 yotta-humanize 的 HZ-01 ~ HZ-24，
/// 修辞动作规则借鉴自开源“去 AI 味”Skill：human-writing / Humanizer-zh / chinese-ai-humanizer）。
struct RuleSpec {
    id: &'static str,
    name: &'static str,
    group: RuleGroup,
    weight: i32,
    words: &'static [&'static str],
    regexes: &'static [&'static str],
    suggestion: &'static str,
}

const RULES: &[RuleSpec] = &[
    RuleSpec {
        id: "HZ-01",
        name: "空泛升华",
        group: RuleGroup::Content,
        weight: 5,
        words: &[
            "标志着", "彰显了", "充分体现了", "意义重大", "里程碑", "历史性",
            "划时代", "深远影响", "开启了新篇章", "奠定了坚实基础",
            "具有重大意义", "起到了重要作用",
        ],
        regexes: &[],
        suggestion: "把空泛升华换成具体事实：谁在何时做了什么、结果如何，而不是“标志着/彰显了”。",
    },
    RuleSpec {
        id: "HZ-02",
        name: "套话开头",
        group: RuleGroup::Content,
        weight: 4,
        words: &[
            "众所周知", "毋庸置疑", "毫无疑问", "在当今", "在当下", "近年来",
            "随着时代", "随着社会",
        ],
        regexes: &[r"随着[^，。；]{0,24}的(?:发展|进步|普及|到来|推进)[，,]"],
        suggestion: "删掉模板化开头，直接从事实或结论进入正文。",
    },
    RuleSpec {
        id: "HZ-03",
        name: "套话结尾",
        group: RuleGroup::Content,
        weight: 4,
        words: &[
            "综上所述", "展望未来", "未来可期", "拭目以待", "共创美好未来",
            "共同书写", "让我们一起", "值得期待",
        ],
        regexes: &[],
        suggestion: "结尾落到具体结论或下一步动作，而不是“展望未来/未来可期”这类空话。",
    },
    RuleSpec {
        id: "HZ-04",
        name: "空话概括",
        group: RuleGroup::Content,
        weight: 3,
        words: &[
            "从某种程度上说", "在某种意义上", "归根结底", "说白了",
            "本质上来说", "实际上来说", "说白了就是", "换句话讲",
        ],
        regexes: &[],
        suggestion: "删掉空话概括；如果这句没有新信息，直接删，不要“说白了”绕一圈。",
    },
    RuleSpec {
        id: "HZ-05",
        name: "模糊归因",
        group: RuleGroup::Content,
        weight: 4,
        words: &[
            "专家指出", "专家认为", "研究表明", "业内人士认为",
            "业内人士指出", "据统计", "有观点认为", "普遍认为",
            "相关数据显示", "调查显示",
        ],
        regexes: &[],
        suggestion: "给出真实来源（哪家机构/哪篇论文/哪份数据），或直接删掉归因。",
    },
    RuleSpec {
        id: "HZ-06",
        name: "绝对化断言",
        group: RuleGroup::Content,
        weight: 3,
        words: &[
            "百分之百", "任何情况下", "绝不可能", "必然导致", "一定会",
            "绝对会", "完全取决于",
        ],
        regexes: &[r"毫无(?:疑问|争议)[，,。]?"],
        suggestion: "把“一定/必然/毫无疑问”改成“通常/往往会/大多数情况下”，给事实留余地。",
    },
    RuleSpec {
        id: "HZ-07",
        name: "AI 黑话",
        group: RuleGroup::Language,
        weight: 5,
        words: &[
            "赋能", "抓手", "闭环", "颗粒度", "链路", "底层逻辑", "顶层设计",
            "方法论", "沉淀", "洞察", "心智", "范式", "发力点", "痛点",
            "落地", "场景化", "维度", "框架", "杠杆", "护城河", "第二曲线",
        ],
        regexes: &[],
        suggestion: "把黑话换成大白话：赋能→支持、抓手→切入点、闭环→流程、链路→流程、底层逻辑→根本原因。",
    },
    RuleSpec {
        id: "HZ-08",
        name: "冗余强调",
        group: RuleGroup::Language,
        weight: 3,
        words: &["非常非常", "十分十分", "极其极其", "特别特别", "真的真的"],
        regexes: &[
            r"(非常|十分|极其|特别|相当|无比|异常|格外)(非常|十分|极其|特别|相当|无比|异常|格外)",
        ],
        suggestion: "删掉叠用的强调词，一句里留一个就好；太多“非常”会让语气失真。",
    },
    RuleSpec {
        id: "HZ-09",
        name: "同义堆叠",
        group: RuleGroup::Language,
        weight: 4,
        words: &[
            "完善和优化", "促进和推动", "帮助和协助", "提高和改善",
            "加强和完善", "规划和部署", "支持和帮助", "鼓励和支持",
            "推动和促进", "改进和完善",
        ],
        regexes: &[],
        suggestion: "同义堆叠留一个动词即可（完善和优化→完善），两个词说的是一件事。",
    },
    RuleSpec {
        id: "HZ-10",
        name: "「我们」滥用",
        group: RuleGroup::Language,
        weight: 3,
        words: &[
            "我们坚信", "我们致力于", "我们始终", "我们深知", "我们相信",
            "我们应该", "我们需要做的", "我们每个人都",
        ],
        regexes: &[],
        suggestion: "“我们坚信/致力于”是宣传腔；改成具体做法或删掉主语直接说事。",
    },
    RuleSpec {
        id: "HZ-11",
        name: "名词化堆砌",
        group: RuleGroup::Language,
        weight: 3,
        words: &[
            "问题的解决", "能力的提升", "水平的增强", "意识的加强",
            "建设的推进", "工作的开展", "质量的提高", "效率的提升",
        ],
        regexes: &[],
        suggestion: "把“XX 的解决/提升/加强”改回动词句：直接说“解决了问题、提升了能力”。",
    },
    RuleSpec {
        id: "HZ-12",
        name: "程度词堆积",
        group: RuleGroup::Language,
        weight: 2,
        words: &[
            "更加深入地", "更加有效地", "更好地", "进一步地", "更为重要",
            "更为关键", "更为突出",
        ],
        regexes: &[],
        suggestion: "程度词一多句子就发飘；删到零或留一个，用具体例子代替“更加”。",
    },
    RuleSpec {
        id: "HZ-13",
        name: "排比三连",
        group: RuleGroup::Style,
        weight: 4,
        words: &[],
        regexes: &[
            r"不仅[^，。]{2,20}，?而且[^，。]{2,20}，?更[^，。]{2,20}",
            r"既[^，。]{2,16}，又[^，。]{2,16}，还[^，。]{2,16}",
        ],
        suggestion: "排比三连是典型 AI 句式；拆成两三句，或只留最有信息量的一项。",
    },
    RuleSpec {
        id: "HZ-14",
        name: "机械列举",
        group: RuleGroup::Style,
        weight: 3,
        words: &[],
        regexes: &[
            r"(首先|其次|再次|最后)[，,]",
            r"(第一|第二|第三|第四)[，,、]",
        ],
        suggestion: "“首先/其次/再次/最后”一长串很机械；按内容自然过渡，别每段都编号。",
    },
    RuleSpec {
        id: "HZ-15",
        name: "反问堆叠",
        group: RuleGroup::Style,
        weight: 2,
        words: &["难道不是吗", "难道不", "怎能", "岂不", "何尝不是"],
        regexes: &[],
        suggestion: "连续反问像在喊口号；留一个反问或改成陈述句更有力。",
    },
    RuleSpec {
        id: "HZ-16",
        name: "破折号滥用",
        group: RuleGroup::Style,
        weight: 3,
        words: &[],
        regexes: &[r"——"],
        suggestion: "破折号全文超过 2 处就显刻意；多余的换成逗号或直接断开句子。",
    },
    RuleSpec {
        id: "HZ-17",
        name: "引号滥用",
        group: RuleGroup::Style,
        weight: 2,
        words: &[],
        regexes: &[],
        suggestion: "给普通词加引号很 AI（“赋能”“闭环”）；去掉引号，或只给真正需要强调的词加。",
    },
    RuleSpec {
        id: "HZ-18",
        name: "感叹号滥用",
        group: RuleGroup::Style,
        weight: 2,
        words: &[],
        regexes: &[r"！"],
        suggestion: "感叹号满屏是营销腔；全文最多留一两个。",
    },
    RuleSpec {
        id: "HZ-19",
        name: "聊天残渣",
        group: RuleGroup::Communication,
        weight: 5,
        words: &[
            "希望对你有所帮助", "希望这些内容对你有用", "如果你有任何问题",
            "如果还有疑问", "随时告诉我", "让我们开始吧", "让我们开始",
            "希望以上内容", "希望这个回答", "欢迎随时提问",
        ],
        regexes: &[],
        suggestion: "删掉“希望对你有所帮助/随时告诉我”这类聊天机器人残渣，正文结尾干净利落。",
    },
    RuleSpec {
        id: "HZ-20",
        name: "奉承讨好",
        group: RuleGroup::Communication,
        weight: 4,
        words: &[
            "很好的问题", "非常好的问题", "你说得对", "你说得很有道理",
            "感谢你的提问", "非常荣幸", "您说得太对了", "好问题",
        ],
        regexes: &[],
        suggestion: "删掉“很好的问题/你说得对”这类奉承开头，直接回答问题。",
    },
    RuleSpec {
        id: "HZ-21",
        name: "免责回避",
        group: RuleGroup::Communication,
        weight: 3,
        words: &[
            "由于篇幅有限", "限于篇幅", "考虑到时间关系", "这里不再赘述",
            "在此不过多展开", "不在此赘述", "受篇幅所限", "因时间有限",
        ],
        regexes: &[],
        suggestion: "“限于篇幅/不再赘述”是 AI 缩水的遮羞布；要么给关键信息，要么别提。",
    },
    RuleSpec {
        id: "HZ-22",
        name: "废话填充",
        group: RuleGroup::Filler,
        weight: 2,
        words: &["换句话说", "换言之", "也就是说", "某种意义上", "某种程度上"],
        regexes: &[r"(其实|就是说)[，,]"],
        suggestion: "“其实/就是说/换句话说”经常是废话；删掉后句子意思不变就删。",
    },
    RuleSpec {
        id: "HZ-23",
        name: "无效转折",
        group: RuleGroup::Filler,
        weight: 2,
        words: &["话说回来", "换个角度", "换个思路", "再说一句", "另外一点"],
        regexes: &[],
        suggestion: "“话说回来/换个角度”是凑字数的转折；直接写下一段内容。",
    },
    RuleSpec {
        id: "HZ-24",
        name: "重复声明",
        group: RuleGroup::Filler,
        weight: 3,
        words: &[
            "值得注意的是", "不难发现", "显而易见", "值得一提的是",
            "需要指出的是", "必须强调的是", "需要强调的是",
            "特别值得注意的是",
        ],
        regexes: &[],
        suggestion: "“值得注意的是/不难发现”是 AI 的提示词残留；删掉，让事实自己说话。",
    },
    // 以下规则借鉴自开源去 AI 味 Skill 的方法论（human-writing / Humanizer-zh 等）：
    // 它们抓的不只是“词”，而是 AI 常见的修辞动作——翻案、排比、抒情、升华。
    RuleSpec {
        id: "HZ-25",
        name: "翻案腔",
        group: RuleGroup::Style,
        weight: 4,
        words: &["回头才发现", "答案恰恰相反"],
        regexes: &[
            r"不是[^。！？；，,]{0,18}(?:，|,)?而是",
            r"并非[^。！？；，,]{0,18}(?:，|,)?而是",
            r"不在于[^。！？；，,]{0,18}(?:，|,)?而在于",
            r"与其说[^。！？；，,]{0,18}(?:，|,)?不如说",
            r"表面[^。！？；，,]{0,18}(?:，|,)?实际(?:上)?",
            r"看似[^。！？；，,]{0,18}(?:，|,)?实则",
            r"你以为[^。！？；，,]{0,18}(?:，|,)?其实",
            r"[^。！？；]{0,16}不重要，重要的是",
            // 情感文常用的变体：“不是A，而是/只是/是B”“不是A，也不是B。是因为C”
            r"不是[^。！？\n，,]{2,24}[，,]\s*(?:而是|只是|才是|就是因为|是)",
            r"(?:不是因为|不是为了)[^。！？\n]{0,40}[，,](?:也不是因为|也不是为了|更不是因为)[^。！？\n]{0,40}[。！？]?[^。！？\n]{0,10}?(?:是因为|而是|只是)",
        ],
        suggestion: "“不是A，而是B/你以为…其实…”以及“不是A，也不是B，是因为C”是典型翻案腔：先立一个读者没有的靶子再推翻，或把答案藏进否定里。改成从正面直接下判断，再放依据。",
    },
    RuleSpec {
        id: "HZ-26",
        name: "抽象抒情词",
        group: RuleGroup::Language,
        weight: 2,
        words: &[
            "安放", "抵达", "微光", "褶皱", "丰盈", "滚烫",
            "轻盈", "赤裸", "剥开", "时代回响", "被好好爱着",
        ],
        regexes: &[],
        suggestion: "抽象概念配具体动词/抒情词（时间保管细节、焦虑显出形状、被好好爱着的感觉）是 AI 抒情腔；落到人的动作与具体事实。",
    },
    RuleSpec {
        id: "HZ-27",
        name: "洞察路标",
        group: RuleGroup::Filler,
        weight: 3,
        words: &[
            "更微妙的是", "还有一层", "只说对了一半", "说穿了，",
            "先说结论，", "更深一层的是", "真正的问题在于",
        ],
        regexes: &[],
        suggestion: "“更微妙的是/还有一层/只说对了一半”是模型给段落抬价的洞察路标，删掉后事实差异自己承担推进。",
    },
    RuleSpec {
        id: "HZ-28",
        name: "金句升华收尾",
        group: RuleGroup::Content,
        weight: 4,
        words: &[],
        regexes: &[
            r"真正的[^。！？]{0,12}从来(?:不|都)",
            r"(?:这就是|这才是|那才是)[^。！？]{0,14}的意义",
            r"(?:爱情|生活|人生|成长)[^。！？]{0,16}(?:的意义|的样子|的本质)",
            r"原来[^。！？]{0,18}才(?:是|会|懂)",
        ],
        suggestion: "结尾像“可引用金句”就是 AI 味：删掉升华式总结，停在实际动作、具体物件或一个未完成的瞬间。",
    },
    RuleSpec {
        id: "HZ-29",
        name: "感悟式升华",
        group: RuleGroup::Content,
        weight: 5,
        words: &[
            "现在你懂了", "你突然明白了", "我终于明白了", "这一刻才明白",
            "这就是爱情", "原来被好好爱着",
        ],
        regexes: &[
            // 回忆叙事里反复出现的“醒悟路标”：现在你懂了/你突然明白了/你后来才懂
            r"(?:你|我|她|他)(?:突然|终于|后来才|现在|这才|总算)?(?:明白|懂了|意识到|看懂了)[了，：:]",
            // 用“真正的X”或“爱情真实的样子”直接给全篇下定义
            r"真正的(?:爱|爱情|生活|人生|成长|幸福|陪伴|浪漫)[^。！？]{0,24}(?:从来|不是|不在|藏在|需要|是)",
            r"(?:爱情|生活|人生|婚姻|幸福|浪漫|爱)[^。！？\n]{0,24}(?:本来就不是|不是用来|不需要|该是|藏在|真实的样子|的意义)",
            // “原来…的感觉/才明白/是这样”，把具体事件拔高成人生感悟
            r"原来[^。！？\n]{0,26}(?:的感觉|的温度|才明白|才懂|是这样)",
            // “就像/正如你们的爱情”，用比喻把故事推广成普遍道理
            r"(?:就像|正如|仿佛)[^。！？\n]{0,18}(?:爱情|生活|人生|婚姻|我们|你们)",
            // “你以为X。但后来你才发现Y” 的觉醒结构
            r"你以为[^。！？\n]{0,70}[。！？][\s]*[^。！？\n]{0,10}?(?:但|可|原来|后来才)",
        ],
        suggestion: "叙事文里频繁出现“你突然明白了/原来…的感觉/真正的爱是…”这类感悟式升华是 AI 煽情模板：把道理留给读者自己体会，删掉一半以上的点题句，停在具体场景和动作上。",
    },
];

/// 确定性机械改写映射（只收“无歧义、改了必更好”的变换；判断级改写交给 LLM）。
struct FixSpec {
    rule_id: &'static str,
    rule_name: &'static str,
    pattern: &'static str,
    replacement: &'static str,
}

const FIXES: &[FixSpec] = &[
    // 空泛升华
    FixSpec {
        rule_id: "HZ-01",
        rule_name: "空泛升华",
        pattern: "充分体现了",
        replacement: "体现了",
    },
    FixSpec {
        rule_id: "HZ-01",
        rule_name: "空泛升华",
        pattern: "具有重大意义",
        replacement: "很重要",
    },
    FixSpec {
        rule_id: "HZ-01",
        rule_name: "空泛升华",
        pattern: "起到了重要作用",
        replacement: "起了作用",
    },
    // 套话开头
    FixSpec {
        rule_id: "HZ-02",
        rule_name: "套话开头",
        pattern: "众所周知，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-02",
        rule_name: "套话开头",
        pattern: "在当今[^，。]{0,12}背景下，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-02",
        rule_name: "套话开头",
        pattern: "在当下，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-02",
        rule_name: "套话开头",
        pattern: "近年来，",
        replacement: "",
    },
    // 套话结尾
    FixSpec {
        rule_id: "HZ-03",
        rule_name: "套话结尾",
        pattern: "综上所述，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-03",
        rule_name: "套话结尾",
        pattern: "总之，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-03",
        rule_name: "套话结尾",
        pattern: "展望未来，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-03",
        rule_name: "套话结尾",
        pattern: "未来可期。",
        replacement: "。",
    },
    FixSpec {
        rule_id: "HZ-03",
        rule_name: "套话结尾",
        pattern: "让我们拭目以待",
        replacement: "拭目以待",
    },
    // 空话概括
    FixSpec {
        rule_id: "HZ-04",
        rule_name: "空话概括",
        pattern: "从某种程度上说，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-04",
        rule_name: "空话概括",
        pattern: "在某种意义上，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-04",
        rule_name: "空话概括",
        pattern: "归根结底，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-04",
        rule_name: "空话概括",
        pattern: "说白了，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-04",
        rule_name: "空话概括",
        pattern: "本质上来说，",
        replacement: "",
    },
    // 模糊归因
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "专家指出，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "专家认为，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "研究表明，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "业内人士认为，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "业内人士指出，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "据统计，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "有观点认为，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "普遍认为，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "相关数据显示，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-05",
        rule_name: "模糊归因",
        pattern: "调查显示，",
        replacement: "",
    },
    // 绝对化断言（软化，不改变大意）
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "毫无疑问，",
        replacement: "显然，",
    },
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "毫无疑问",
        replacement: "显然",
    },
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "一定会",
        replacement: "往往会",
    },
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "绝对会",
        replacement: "往往会",
    },
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "必然导致",
        replacement: "往往导致",
    },
    FixSpec {
        rule_id: "HZ-06",
        rule_name: "绝对化断言",
        pattern: "百分之百",
        replacement: "基本",
    },
    // AI 黑话 → 大白话
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "赋能",
        replacement: "支持",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "抓手",
        replacement: "切入点",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "闭环",
        replacement: "流程",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "颗粒度",
        replacement: "细致程度",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "链路",
        replacement: "流程",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "底层逻辑",
        replacement: "根本原因",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "顶层设计",
        replacement: "整体规划",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "方法论",
        replacement: "方法",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "沉淀",
        replacement: "积累",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "洞察",
        replacement: "发现",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "心智",
        replacement: "观念",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "范式",
        replacement: "模式",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "发力点",
        replacement: "重点",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "痛点",
        replacement: "难点",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "落地",
        replacement: "落实",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "场景化",
        replacement: "按情境",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "维度",
        replacement: "角度",
    },
    FixSpec {
        rule_id: "HZ-07",
        rule_name: "AI 黑话",
        pattern: "框架",
        replacement: "结构",
    },
    // 冗余强调
    FixSpec {
        rule_id: "HZ-08",
        rule_name: "冗余强调",
        pattern: "非常非常",
        replacement: "非常",
    },
    FixSpec {
        rule_id: "HZ-08",
        rule_name: "冗余强调",
        pattern: "十分十分",
        replacement: "十分",
    },
    FixSpec {
        rule_id: "HZ-08",
        rule_name: "冗余强调",
        pattern: "极其极其",
        replacement: "极其",
    },
    FixSpec {
        rule_id: "HZ-08",
        rule_name: "冗余强调",
        pattern: "特别特别",
        replacement: "特别",
    },
    FixSpec {
        rule_id: "HZ-08",
        rule_name: "冗余强调",
        pattern: "真的真的",
        replacement: "真的",
    },
    // 同义堆叠
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "完善和优化",
        replacement: "完善",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "促进和推动",
        replacement: "推动",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "帮助和协助",
        replacement: "帮助",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "提高和改善",
        replacement: "改进",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "加强和完善",
        replacement: "完善",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "规划和部署",
        replacement: "规划",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "支持和帮助",
        replacement: "帮助",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "鼓励和支持",
        replacement: "支持",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "推动和促进",
        replacement: "推动",
    },
    FixSpec {
        rule_id: "HZ-09",
        rule_name: "同义堆叠",
        pattern: "改进和完善",
        replacement: "完善",
    },
    // “我们”滥用
    FixSpec {
        rule_id: "HZ-10",
        rule_name: "「我们」滥用",
        pattern: "我们坚信",
        replacement: "我们认为",
    },
    FixSpec {
        rule_id: "HZ-10",
        rule_name: "「我们」滥用",
        pattern: "我们致力于",
        replacement: "我们在做",
    },
    FixSpec {
        rule_id: "HZ-10",
        rule_name: "「我们」滥用",
        pattern: "我们深知",
        replacement: "我们知道",
    },
    // 名词化堆砌
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "问题的解决",
        replacement: "解决问题",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "能力的提升",
        replacement: "提升能力",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "水平的增强",
        replacement: "增强水平",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "意识的加强",
        replacement: "加强意识",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "建设的推进",
        replacement: "推进建设",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "工作的开展",
        replacement: "开展工作",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "质量的提高",
        replacement: "提高质量",
    },
    FixSpec {
        rule_id: "HZ-11",
        rule_name: "名词化堆砌",
        pattern: "效率的提升",
        replacement: "提升效率",
    },
    // 程度词堆积
    FixSpec {
        rule_id: "HZ-12",
        rule_name: "程度词堆积",
        pattern: "更加深入地",
        replacement: "深入地",
    },
    FixSpec {
        rule_id: "HZ-12",
        rule_name: "程度词堆积",
        pattern: "更加有效地",
        replacement: "有效地",
    },
    FixSpec {
        rule_id: "HZ-12",
        rule_name: "程度词堆积",
        pattern: "进一步地",
        replacement: "进一步",
    },
    FixSpec {
        rule_id: "HZ-12",
        rule_name: "程度词堆积",
        pattern: "更为重要",
        replacement: "更重要",
    },
    // 聊天残渣
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "希望对你有所帮助。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "希望这些内容对你有用。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "如果你有任何问题，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "如果还有疑问，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "随时告诉我。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "让我们开始吧。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "让我们开始。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "希望以上内容对你有用。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-19",
        rule_name: "聊天残渣",
        pattern: "欢迎随时提问。",
        replacement: "",
    },
    // 奉承讨好
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "很好的问题。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "非常好的问题。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "你说得对，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "你说得很有道理，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "感谢你的提问。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "非常荣幸",
        replacement: "很荣幸",
    },
    FixSpec {
        rule_id: "HZ-20",
        rule_name: "奉承讨好",
        pattern: "您说得太对了。",
        replacement: "",
    },
    // 免责回避
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "由于篇幅有限，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "限于篇幅，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "考虑到时间关系，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "这里不再赘述。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "在此不过多展开。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "不在此赘述。",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "受篇幅所限，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-21",
        rule_name: "免责回避",
        pattern: "因时间有限，",
        replacement: "",
    },
    // 废话填充
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "换句话说，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "换言之，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "也就是说，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "某种意义上，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "某种程度上，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "其实，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-22",
        rule_name: "废话填充",
        pattern: "就是说，",
        replacement: "",
    },
    // 无效转折
    FixSpec {
        rule_id: "HZ-23",
        rule_name: "无效转折",
        pattern: "话说回来，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-23",
        rule_name: "无效转折",
        pattern: "换个角度，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-23",
        rule_name: "无效转折",
        pattern: "换个思路，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-23",
        rule_name: "无效转折",
        pattern: "再说一句，",
        replacement: "",
    },
    // 重复声明
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "值得注意的是，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "不难发现，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "显而易见，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "值得一提的是，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "需要指出的是，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "必须强调的是，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "需要强调的是，",
        replacement: "",
    },
    FixSpec {
        rule_id: "HZ-24",
        rule_name: "重复声明",
        pattern: "特别值得注意的是，",
        replacement: "",
    },
];

/// 一条规则命中的摘要
#[derive(Debug, Clone)]
pub struct RuleHit {
    pub rule_id: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub group_label: &'static str,
    pub weight: i32,
    pub count: usize,
    pub samples: Vec<String>,
    pub suggestion: &'static str,
}

/// 一次确定性改写的修复记录
#[derive(Debug, Clone)]
pub struct DeterministicFix {
    pub rule_id: &'static str,
    pub rule_name: &'static str,
    pub count: usize,
    pub detail: String,
}

/// 确定性机械改写的完整结果
#[derive(Debug, Clone)]
pub struct DeterministicRewrite {
    pub text: String,
    pub fixes: Vec<DeterministicFix>,
    pub before_score: i64,
    pub after_score: i64,
}

#[derive(Debug, Clone)]
pub struct LocalFeatureReport {
    /// 0-100，越大越像 AI
    pub score: i64,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub sentence_len_cv: Option<f64>,
    pub paragraph_len_cv: Option<f64>,
    pub short_sentence_ratio: f64,
    pub connector_density: f64,
    pub ai_phrase_hits: usize,
    pub mattr: Option<f64>,
    /// 本地 AI 腔规则子分（0-100，越高越像）
    pub rule_score: i64,
    /// 规则命中总处数
    pub rule_hit_count: usize,
    /// 命中的规则种类数
    pub rule_types: usize,
    /// 命中的规则明细
    pub rule_hits: Vec<RuleHit>,
    /// 命中的具体问题，供前端/LLM 展示
    pub signals: Vec<String>,
}

fn visible_len(s: &str) -> usize {
    s.chars().filter(|c| !c.is_whitespace()).count()
}

fn split_sentences(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (idx, ch) in content.char_indices() {
        if SENTENCE_END.contains(&ch) {
            let seg = content[start..idx].trim();
            if !seg.is_empty() {
                out.push(seg.to_string());
            }
            start = idx + ch.len_utf8();
        }
    }
    let tail = content[start..].trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn split_paragraphs(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn mean_std(values: &[usize]) -> Option<(f64, f64)> {
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().sum::<usize>() as f64 / values.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let variance = values
        .iter()
        .map(|v| {
            let d = *v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / values.len() as f64;
    Some((mean, variance.sqrt()))
}

fn coefficient_of_variation(values: &[usize]) -> Option<f64> {
    let (mean, sd) = mean_std(values)?;
    Some(sd / mean)
}

fn count_occurrences(text: &str, needles: &[&str]) -> usize {
    let mut total = 0usize;
    for needle in needles {
        let mut rest = text;
        while let Some(pos) = rest.find(needle) {
            total += 1;
            let next = pos + needle.len();
            if next >= rest.len() {
                break;
            }
            rest = &rest[next..];
        }
    }
    total
}

fn has_ordered_sequence(text: &str, needles: &[&str]) -> bool {
    let mut rest = text;
    for (idx, needle) in needles.iter().enumerate() {
        match rest.find(needle) {
            Some(pos) => {
                if idx + 1 == needles.len() {
                    return true;
                }
                let next = pos + needle.len();
                if next >= rest.len() {
                    return false;
                }
                rest = &rest[next..];
            }
            None => return false,
        }
    }
    true
}

/// Moving Average Type-Token Ratio：反映用字多样性，越低越像“字汇被收敛”的机器文。
fn mattr(content: &str) -> Option<f64> {
    let chars: Vec<char> = content.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.len() < MATTR_WINDOW {
        return None;
    }
    let mut total = 0.0f64;
    let mut windows = 0usize;
    let mut i = 0usize;
    while i + MATTR_WINDOW <= chars.len() {
        let window = &chars[i..i + MATTR_WINDOW];
        let unique = window.iter().copied().collect::<HashSet<char>>().len();
        total += unique as f64 / MATTR_WINDOW as f64;
        windows += 1;
        i += MATTR_STEP;
    }
    if windows == 0 {
        None
    } else {
        Some(total / windows as f64)
    }
}

fn connector_density(content: &str, total_chars: usize) -> f64 {
    if total_chars == 0 {
        return 0.0;
    }
    count_occurrences(content, AI_CONNECTORS) as f64 * 1000.0 / total_chars as f64
}

/// 编译后的规则（词表已转义成正则）。
struct CompiledRule {
    spec: &'static RuleSpec,
    patterns: Vec<Regex>,
}

fn compiled_rules() -> &'static Vec<CompiledRule> {
    static COMPILED: OnceLock<Vec<CompiledRule>> = OnceLock::new();
    COMPILED.get_or_init(|| {
        RULES
            .iter()
            .map(|spec| {
                let mut patterns = Vec::new();
                for w in spec.words {
                    patterns.push(Regex::new(&regex::escape(w)).expect("static word regex"));
                }
                for rx in spec.regexes {
                    patterns.push(Regex::new(rx).expect("static rule regex"));
                }
                CompiledRule { spec, patterns }
            })
            .collect()
    })
}

/// 运行全部 AI 腔词表/句式规则，返回命中明细。
fn detect_rule_hits(text: &str) -> Vec<RuleHit> {
    let mut hits = Vec::new();
    for compiled in compiled_rules() {
        let spec = compiled.spec;
        let mut count = 0usize;
        let mut samples: Vec<String> = Vec::new();
        if spec.id == "HZ-17" {
            // 引号规则只盯“给普通词加引号”，对话/引述里的引号不参与，避免文学性误报。
            let (sc, ss) = suspicious_quote_hits(text);
            count = sc;
            samples = ss;
        } else {
            for pat in &compiled.patterns {
                for m in pat.find_iter(text) {
                    count += 1;
                    if samples.len() < 3 {
                        let matched: String = m.as_str().chars().take(24).collect();
                        samples.push(matched);
                    }
                }
            }
        }
        // 标点/结构类规则要成规模才算“命中”，避免把正常引用标点误判成 AI 腔
        let min_hits = match spec.id {
            "HZ-16" | "HZ-18" => 3,
            "HZ-17" => 2,
            "HZ-14" => 2,
            // 单次“不是A，而是B”是正常解释；连用成套路才算翻案腔
            "HZ-25" => 2,
            _ => 1,
        };
        if count >= min_hits {
            hits.push(RuleHit {
                rule_id: spec.id,
                name: spec.name,
                group: spec.group.code(),
                group_label: spec.group.label(),
                weight: spec.weight,
                count,
                samples,
                suggestion: spec.suggestion,
            });
        }
    }
    hits
}

/// 统计“给普通词加引号”式的引号用法，跳过对话/心理活动引语。
fn suspicious_quote_hits(text: &str) -> (usize, Vec<String>) {
    static QUOTE: OnceLock<Regex> = OnceLock::new();
    static MARKER: OnceLock<Regex> = OnceLock::new();
    let quote_re = QUOTE.get_or_init(|| Regex::new(r"“([^”]{1,14})”").expect("quote regex"));
    let marker_re = MARKER.get_or_init(|| {
        Regex::new(
            r"(说|道|问|答|想|觉得|喊|叫|嚷|写|复述|认为|告诉|提醒|嘀咕|称|表示|回|补|句|念|读)",
        )
        .expect("marker regex")
    });

    let mut count = 0usize;
    let mut samples = Vec::new();
    for caps in quote_re.captures_iter(text) {
        let whole = caps.get(0).expect("quote whole");
        let inner = caps.get(1).expect("quote inner").as_str();
        if inner.chars().any(|c| {
            matches!(
                c,
                '，' | '。' | '、' | '；' | '：' | '？' | '！' | '…' | '—' | ','
            )
        }) {
            continue;
        }
        let inner_len = inner.chars().count();
        if !(2..=8).contains(&inner_len) {
            continue;
        }
        // 引号前若有“说/道/觉得/写/复述”等提示词，视为正常引语
        let before: String = text[..whole.start()].chars().rev().take(6).collect();
        let ctx: String = before.chars().rev().collect();
        if marker_re.is_match(&ctx) {
            continue;
        }
        count += 1;
        if samples.len() < 3 {
            samples.push(whole.as_str().chars().take(24).collect());
        }
    }
    (count, samples)
}

/// 规则分：加权命中密度 + 规则种类广度 + 分组多样性（0-100）。
fn rule_component_score(hits: &[RuleHit], char_count: usize) -> i64 {
    if hits.is_empty() || char_count == 0 {
        return 0;
    }
    let weighted: f64 = hits
        .iter()
        .map(|h| (h.weight as f64) * (h.count as f64))
        .sum();
    let density = weighted / char_count as f64 * 100.0;
    let density_score = ((density + 1.0).log2() * 14.0).min(60.0);
    let breadth = (hits.len() * 2).min(20) as f64;
    let groups = hits.iter().map(|h| h.group).collect::<HashSet<_>>().len();
    let group_score = (groups * 4).min(16) as f64;
    ((density_score + breadth + group_score).min(100.0)).round() as i64
}

/// 把规则命中转成给用户/LLM 看的诊断语句（只列权重高或成规模的）。
fn rule_signal_lines(hits: &[RuleHit]) -> Vec<String> {
    if hits.is_empty() {
        return Vec::new();
    }
    let mut sorted = hits.to_vec();
    sorted.sort_by(|a, b| {
        (b.weight * b.count as i32)
            .cmp(&(a.weight * a.count as i32))
            .then_with(|| a.name.cmp(b.name))
    });
    let mut lines = Vec::new();
    for h in sorted.iter().take(12) {
        if h.weight < 4 && h.count < 2 {
            continue;
        }
        let sample = if h.samples.is_empty() {
            String::new()
        } else {
            format!(
                "（{}）",
                h.samples
                    .iter()
                    .map(|s| format!("“{}”", s))
                    .collect::<Vec<_>>()
                    .join("、")
            )
        };
        lines.push(format!(
            "[{}] 命中「{}」×{}：{}——{}",
            h.group_label, h.name, h.count, sample, h.suggestion
        ));
    }
    if sorted.len() > 12 {
        lines.push(format!(
            "……另有 {} 类规则有轻微命中，未逐条列出。",
            sorted.len() - 12
        ));
    }
    lines
}

fn limit_dash(text: &str) -> (String, usize) {
    let count = text.matches("——").count();
    if count <= 2 {
        return (text.to_string(), 0);
    }
    let mut out = String::with_capacity(text.len());
    let mut seen = 0usize;
    let mut i = 0usize;
    let mut changed = 0usize;
    let chars: Vec<char> = text.chars().collect();
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '—' && chars[i + 1] == '—' {
            if seen >= 2 {
                out.push('，');
                changed += 1;
            } else {
                out.push('—');
                out.push('—');
            }
            seen += 1;
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    (out, changed)
}

fn limit_bang(text: &str) -> (String, usize) {
    let count = text.matches('！').count();
    if count <= 2 {
        return (text.to_string(), 0);
    }
    let mut out = String::with_capacity(text.len());
    let mut seen = 0usize;
    let mut changed = 0usize;
    for ch in text.chars() {
        if ch == '！' {
            if seen >= 2 {
                out.push('。');
                changed += 1;
            } else {
                out.push(ch);
            }
            seen += 1;
        } else {
            out.push(ch);
        }
    }
    (out, changed)
}

fn trim_blank_punct(text: &str) -> (String, usize) {
    let mut out = text.to_string();
    let mut changed = 0usize;
    for (pat, rep) in [
        (r"[，,]{2,}", "，"),
        (r"[。]{2,}", "。"),
        (r"[，,][。]", "。"),
    ] {
        if let Ok(re) = Regex::new(pat) {
            let n = re.find_iter(&out).count();
            if n > 0 {
                out = re.replace_all(&out, rep).to_string();
                changed += n;
            }
        }
    }
    if let Ok(re) = Regex::new(r"^[，,。；;\s]+") {
        if re.is_match(&out) {
            out = re.replace(&out, "").to_string();
            changed += 1;
        }
    }
    (out, changed)
}

/// 确定性机械改写：替换 AI 黑话、删除套话/残渣、标点限流。
/// 只做“改了必更好”的变换，判断级润色仍交给 LLM；结果可人工复核。
pub fn deterministic_rewrite(content: &str) -> DeterministicRewrite {
    let before_score = scan(content).score;
    let mut out = content.to_string();
    let mut fixes: Vec<DeterministicFix> = Vec::new();
    for fix in FIXES {
        match Regex::new(fix.pattern) {
            Ok(re) => {
                let count = re.find_iter(&out).count();
                if count > 0 {
                    out = re.replace_all(&out, fix.replacement).to_string();
                    let detail = if fix.replacement.is_empty() {
                        format!("删除“{}”", fix.pattern)
                    } else {
                        format!("“{}”→“{}”", fix.pattern, fix.replacement)
                    };
                    fixes.push(DeterministicFix {
                        rule_id: fix.rule_id,
                        rule_name: fix.rule_name,
                        count,
                        detail,
                    });
                }
            }
            Err(_) => continue,
        }
    }

    let (after_dash, dash_n) = limit_dash(&out);
    if dash_n > 0 {
        out = after_dash;
        fixes.push(DeterministicFix {
            rule_id: "HZ-16",
            rule_name: "破折号滥用",
            count: dash_n,
            detail: "多余破折号→逗号".to_string(),
        });
    }
    let (after_bang, bang_n) = limit_bang(&out);
    if bang_n > 0 {
        out = after_bang;
        fixes.push(DeterministicFix {
            rule_id: "HZ-18",
            rule_name: "感叹号滥用",
            count: bang_n,
            detail: "多余感叹号→句号".to_string(),
        });
    }
    let (after_clean, clean_n) = trim_blank_punct(&out);
    if clean_n > 0 {
        out = after_clean;
        fixes.push(DeterministicFix {
            rule_id: "HZ-22",
            rule_name: "废话填充",
            count: clean_n,
            detail: "清理空壳标点".to_string(),
        });
    }

    let after_score = scan(&out).score;
    DeterministicRewrite {
        text: out,
        fixes,
        before_score,
        after_score,
    }
}

pub fn scan(content: &str) -> LocalFeatureReport {
    let total_chars = visible_len(content).max(1);
    let sentences = split_sentences(content);
    let paragraphs = split_paragraphs(content);
    let sentence_lens: Vec<usize> = sentences.iter().map(|s| visible_len(s)).collect();
    let paragraph_lens: Vec<usize> = paragraphs.iter().map(|p| visible_len(p)).collect();
    let sentence_cv = coefficient_of_variation(&sentence_lens);
    let paragraph_cv = coefficient_of_variation(&paragraph_lens);

    let short_count = sentence_lens.iter().filter(|&&len| len <= 10).count();
    let short_ratio = if sentence_lens.is_empty() {
        0.0
    } else {
        short_count as f64 / sentence_lens.len() as f64
    };

    let connectors_per_1000 = connector_density(content, total_chars);
    let ai_phrase_hits = count_occurrences(content, AI_PHRASES);
    let mattr_value = mattr(content);

    let mut stats_score = 0f64;
    let mut signals: Vec<String> = Vec::new();

    // 1) 句长变异系数：AI 文本的句长往往异常均匀
    if let Some(cv) = sentence_cv {
        if cv < 0.40 {
            stats_score += 30.0;
            signals.push(format!(
                "句长分布过于均匀：变异系数仅 {:.2}（低于 0.40），缺少真人写作的长短交错。",
                cv
            ));
        } else if cv < 0.55 {
            stats_score += 18.0;
            signals.push(format!(
                "句长节奏偏均匀：变异系数 {:.2}，接近机器写作的平滑句式。",
                cv
            ));
        } else if cv < 0.70 {
            stats_score += 8.0;
        }
    }

    // 2) 段落长度均匀度
    if paragraphs.len() >= 5 {
        if let Some(cv) = paragraph_cv {
            if cv < 0.45 {
                stats_score += 6.0;
                signals.push(format!(
                    "段落长短高度一致（变异系数 {:.2}），像是按固定篇幅模板切出来的。",
                    cv
                ));
            }
        }
    }

    // 3) 短句缺失：真人写作很少整篇没有 10 字以内的短句
    if sentence_lens.len() >= 5 {
        if short_ratio < 0.05 {
            stats_score += 10.0;
            signals.push(format!(
                "短句比例过低：仅 {:.0}% 的句子在 10 字以内，缺少口语化的停顿与呼吸感。",
                short_ratio * 100.0
            ));
        } else if short_ratio < 0.12 {
            stats_score += 4.0;
        } else if short_ratio >= 0.25 {
            stats_score = (stats_score - 6.0).max(0.0);
        }
    }

    // 4) 连接词/过渡词密度
    if connectors_per_1000 >= 8.0 {
        stats_score += 20.0;
        signals.push(format!(
            "过渡连接词密度偏高：约 {:.1} 次/千字，存在“首先/其次/此外/综上所述”式机械衔接。",
            connectors_per_1000
        ));
    } else if connectors_per_1000 >= 5.0 {
        stats_score += 12.0;
        signals.push(format!(
            "过渡连接词较密：约 {:.1} 次/千字，段落推进依赖模板连接词。",
            connectors_per_1000
        ));
    } else if connectors_per_1000 >= 2.5 {
        stats_score += 5.0;
    }

    // 5) AI/营销高频词
    if ai_phrase_hits >= 6 {
        stats_score += 20.0;
        signals.push(format!(
            "命中 {} 处 AI/营销高频表达（底层逻辑、赋能、总而言之等），词面有浓重模板腔。",
            ai_phrase_hits
        ));
    } else if ai_phrase_hits >= 4 {
        stats_score += 15.0;
        signals.push(format!(
            "命中 {} 处 AI/营销高频表达，需要重点复查这些位置。",
            ai_phrase_hits
        ));
    } else if ai_phrase_hits >= 2 {
        stats_score += 8.0;
    } else if ai_phrase_hits == 1 {
        stats_score += 3.0;
    }

    // 6) 模板化结构：首先…其次…最后 / 一方面…另一方面 / 结尾总结腔
    if has_ordered_sequence(content, &["首先", "其次", "最后"]) {
        stats_score += 15.0;
        signals.push("出现完整“首先…其次…最后”推进链，是高度模板化的分点结构。".to_string());
    } else if has_ordered_sequence(content, &["一方面", "另一方面"]) {
        stats_score += 12.0;
        signals.push("出现“一方面…另一方面”对仗结构，机械并列感明显。".to_string());
    } else if content.contains("综上所述")
        || content.contains("总而言之")
        || content.contains("总的来说")
        || content.contains("总之")
    {
        stats_score += 8.0;
        signals.push("结尾出现“综上所述/总而言之/总之”式总结腔。".to_string());
    }

    // 7) 用字多样性（MATTR）
    if let Some(m) = mattr_value {
        if m < 0.55 {
            stats_score += 12.0;
            signals.push(format!(
                "用字多样性偏低：MATTR={:.2}，全文用词过于收敛。",
                m
            ));
        } else if m < 0.65 {
            stats_score += 7.0;
            signals.push(format!(
                "用字多样性一般：MATTR={:.2}，接近低多样性的机器文本区间。",
                m
            ));
        } else if m < 0.75 {
            stats_score += 3.0;
        }
    }

    // 8) 本地 AI 腔规则（词表/句式/修辞动作/标点滥用）
    let rule_hits = detect_rule_hits(content);
    let rule_score = rule_component_score(&rule_hits, total_chars);
    let rule_hit_count: usize = rule_hits.iter().map(|h| h.count).sum();
    let rule_types = rule_hits.len();
    signals.extend(rule_signal_lines(&rule_hits));

    // 规则命中时抬升综合指数；统计信号单独命中时维持原有敏感度，避免回归。
    let stats_score = (stats_score.round() as i64).clamp(0, 100);
    let final_score = if rule_hits.is_empty() {
        stats_score
    } else {
        let blend = (stats_score as f64 * 0.55 + rule_score as f64 * 0.45).round() as i64;
        stats_score.max(blend)
    };

    if signals.is_empty() {
        signals.push("本地规则扫描未发现明显模板/节奏异常，该维度不额外扣分。".to_string());
    }

    LocalFeatureReport {
        score: final_score.clamp(0, 100),
        sentence_count: sentences.len(),
        paragraph_count: paragraphs.len(),
        sentence_len_cv: sentence_cv,
        paragraph_len_cv: paragraph_cv,
        short_sentence_ratio: short_ratio,
        connector_density: connectors_per_1000,
        ai_phrase_hits,
        mattr: mattr_value,
        rule_score,
        rule_hit_count,
        rule_types,
        rule_hits,
        signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_chinese_sentences() {
        let text = "第一句。第二句！第三句？\n第四句";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 4);
        assert_eq!(sentences[0], "第一句");
        assert_eq!(sentences[3], "第四句");
    }

    #[test]
    fn detects_ordered_template() {
        assert!(has_ordered_sequence(
            "首先我们看A，其次看B，最后看C。",
            &["首先", "其次", "最后"]
        ));
        assert!(!has_ordered_sequence(
            "首先只有一处。",
            &["首先", "其次", "最后"]
        ));
    }

    #[test]
    fn template_like_text_scores_higher_than_humanish_text() {
        let template = "首先，人工智能正在改变我们的生活方式。其次，它显著提高了工作效率。\
最后，我们应该积极拥抱这一趋势。综上所述，人工智能的意义十分深远。\
值得注意的是，我们需要理性看待技术发展。总而言之，未来已来。";
        let humanish = "早上七点，我在公交车上刷到一条消息，愣了半天。\
朋友发来的，说他们公司上个月裁了一整层。我回了一句：那你还敢跳槽吗？\
他隔了十分钟才回，说已经在收拾工位了。其实我们都清楚，行业热闹了三年，现在开始还债了。";
        let t = scan(template);
        let h = scan(humanish);
        assert!(
            t.score > h.score,
            "模板文得分 {} 应高于真人感文本得分 {}",
            t.score,
            h.score
        );
        assert!(t.rule_hit_count > h.rule_hit_count);
        assert!(!t.signals.is_empty());
    }

    #[test]
    fn rule_table_catches_typical_ai_words() {
        let text = "企业想要真正实现数字化，首先需要找到抓手，其次形成业务闭环，\
最后沉淀出可复用的方法论。赋能团队、洞察用户痛点、打造生态链路，这些都是顶层设计的一环。";
        let r = scan(text);
        assert!(r.rule_hit_count >= 5, "命中 {} 处不足", r.rule_hit_count);
        assert!(
            r.rule_hits.iter().any(|h| h.name == "AI 黑话"),
            "应命中 AI 黑话规则"
        );
    }

    #[test]
    fn deterministic_rewrite_removes_ai_flavor_words() {
        let original = "众所周知，数字化转型赋能企业发展。综上所述，未来可期。\
希望对你有所帮助。";
        let out = deterministic_rewrite(original);
        assert!(!out.text.contains("众所周知"));
        assert!(!out.text.contains("赋能"));
        assert!(!out.text.contains("综上所述"));
        assert!(!out.text.contains("未来可期"));
        assert!(!out.text.contains("希望对你有所帮助"));
        assert!(!out.fixes.is_empty());
        assert!(out.after_score <= out.before_score);
    }

    #[test]
    fn punctuation_limiter_works() {
        let (a, n) = limit_dash("一——二——三——四");
        assert_eq!(n, 1);
        assert!(!a.contains("四——"));
        let (b, m) = limit_bang("好！好！好！好！");
        assert_eq!(m, 2);
        assert_eq!(b.matches('！').count(), 2);
    }

    #[test]
    fn quote_rule_skips_dialogue_but_keeps_buzzword_quotes() {
        let dialogue = "他说：“谢谢你。”她心里想：“就是今天了。”朋友复述道：“别紧张。”";
        let (n1, _) = suspicious_quote_hits(dialogue);
        assert_eq!(n1, 0, "对话/心理引号不应计入：{}", n1);

        let buzz = "这不仅是“赋能”，还要形成“闭环”，抓住“抓手”。";
        let (n2, samples) = suspicious_quote_hits(buzz);
        assert_eq!(n2, 3, "给普通词加引号应计入");
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn rhetorical_rules_catch_emotional_ai_essay_but_skip_plain_narrative() {
        // 典型 AI 情感文修辞动作：否定反衬 + 醒悟路标 + 把具体故事拔高成人生道理
        let ai_ish = "你不是因为他多完美，而是因为他把你放在心上。\
你点头不是因为他跪得够久，也不是因为那枚戒指在灯光下闪得好看，是因为他把选择权留给了你。\
你突然明白了：真正的爱，从来不在盛大的场景里，而在一杯热牛奶的温度里。\
原来被好好爱着的感觉，是连他手心常年不退的温热都会让你想起。\
就像你们的爱情，不够鲜艳，不够贵重，但风怎么吹，它都不倒。";
        let r = scan(ai_ish);
        assert!(
            r.rule_hits.iter().any(|h| h.rule_id == "HZ-25"),
            "应命中“翻案腔”：{:?}",
            r.rule_hits
        );
        assert!(
            r.rule_hits.iter().any(|h| h.rule_id == "HZ-28"),
            "应命中“金句升华收尾”：{:?}",
            r.rule_hits
        );
        assert!(
            r.rule_hits.iter().any(|h| h.rule_id == "HZ-29"),
            "应命中“感悟式升华”：{:?}",
            r.rule_hits
        );

        // 朴素真人叙述：只有具体动作，不做意义拔高
        let plain = "早上我去便利店买豆浆，翻遍口袋发现忘了带钱。\
老板正在给客人炸油条，头也没抬，说下次一起给。\
我把豆浆抱在怀里往回跑，到家才想起忘了说谢谢。\
第二天中午我又去了一趟，趁他转身拿货，把钱压在他放零钱的铁盒下面。";
        let p = scan(plain);
        assert!(
            !p.rule_hits.iter().any(|h| h.rule_id == "HZ-29"),
            "朴素叙述不应命中“感悟式升华”：{:?}",
            p.rule_hits
        );
        assert!(
            p.rule_hit_count < 4,
            "朴素叙述命中应很少：{:?}",
            p.rule_hits
        );
    }
}
