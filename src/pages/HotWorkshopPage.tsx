/**
 * 爆文工坊：粘贴爆文链接 → 抓取正文 → AI 拆解爆款要素 → 写同款 → 存入内容工作台。
 * 第一期：单链接流程；历史记录存 localStorage。
 */
import { useState } from 'react'
import {
  Alert, Button, Card, Collapse, Input, InputNumber, Select, Space, Spin, Steps, Switch, Tabs, Typography, message,
} from 'antd'
import {
  CopyOutlined, EditOutlined, FileSearchOutlined, FireOutlined, SaveOutlined,
} from '@ant-design/icons'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { marked } from 'marked'
import HotRadarPanel from '../components/hot/HotRadarPanel'
import { getErrorMessage } from '../utils/getErrorMessage'

const { Title, Text, Paragraph } = Typography

const HISTORY_KEY = 'hot-workshop-history'
const CONTENT_LIMIT = 8000

interface WechatArticle {
  title: string
  account: string
  author: string
  publish_time: string
  cover: string
  content_text: string
}

interface HistoryItem {
  url: string
  title: string
  time: number
}

interface Knobs {
  hook?: string
  emotion?: string
  colloquial?: boolean
  cta?: boolean
}

interface QualityGateResult {
  title: string
  content: string
  report: string
  score: number
  rounds: number
}

const HOOK_OPTIONS = [
  { value: 'story', label: '故事场景开场' },
  { value: 'question', label: '提问式开场' },
  { value: 'counterintuitive', label: '反常识开场' },
]

const EMOTION_OPTIONS = [
  { value: 'restrained', label: '克制' },
  { value: 'passionate', label: '热烈' },
]

function loadHistory(): HistoryItem[] {
  try {
    return JSON.parse(localStorage.getItem(HISTORY_KEY) || '[]')
  } catch {
    return []
  }
}

function saveHistory(url: string, title: string) {
  const list = loadHistory().filter((h) => h.url !== url)
  list.unshift({ url, title, time: Date.now() })
  localStorage.setItem(HISTORY_KEY, JSON.stringify(list.slice(0, 30)))
}

const ANALYZE_SYSTEM = `你是一名深谙微信公众号传播规律的资深内容分析师，擅长拆解爆款文章的底层逻辑。
请基于给定文章输出一份结构化拆解报告（Markdown），包含以下小节：
## 一句话概括
## 爆款要素分析
- 标题公式（为什么这个标题让人想点）
- 选题与目标人群
- 开头钩子（前 3 句如何留住读者）
- 结构与节奏（分段逻辑、小标题特点）
- 情绪与共鸣点（哪些段落制造了转发欲）
- 结尾转化
## 可迁移的写作模板
（把上述要素抽象成一份可直接套用的写作模板）
要求：语言精炼，要点需引用原文片段作为依据；不要复述全文。`

const WRITE_SYSTEM = `你是一名百万粉公众号资深写手。请基于提供的爆款拆解报告和用户的选题要求，创作一篇全新的原创文章。
硬性要求：
1. 只借鉴原文的结构、节奏与风格，严禁复用原文的句子、案例和独特事实表述，确保内容原创；
2. 符合微信公众号排版习惯：短段落、适当使用小标题；
3. 输出格式：第一行是文章标题（不要书名号、不要"标题："前缀），随后空一行，输出正文 Markdown。
4. 正文不要出现任何对原文章或拆解报告的引用字样。`

export default function HotWorkshopPage() {
  const [url, setUrl] = useState('')
  const [article, setArticle] = useState<WechatArticle | null>(null)
  const [fetching, setFetching] = useState(false)
  const [analysis, setAnalysis] = useState('')
  const [analyzing, setAnalyzing] = useState(false)
  const [topic, setTopic] = useState('')
  const [wordCount, setWordCount] = useState<number>(1500)
  const [style, setStyle] = useState('')
  const [generated, setGenerated] = useState<{ title: string; content: string } | null>(null)
  const [quality, setQuality] = useState<{ report: string; score: number; rounds: number } | null>(null)
  const [knobs, setKnobs] = useState<Knobs>({})
  const [savingPersona, setSavingPersona] = useState(false)
  const [writing, setWriting] = useState(false)
  const [saving, setSaving] = useState(false)
  const [history, setHistory] = useState<HistoryItem[]>(loadHistory)
  const [activeTab, setActiveTab] = useState('write')

  // 步骤推进
  const step = generated ? 2 : analysis ? 1 : article ? 0 : -1

  const handleFetch = async (targetUrl?: string) => {
    const u = (targetUrl || url).trim()
    if (!u) {
      message.warning('请输入爆文链接')
      return
    }
    setFetching(true)
    setArticle(null)
    setAnalysis('')
    setGenerated(null)
    try {
      const art = await tauriInvoke<WechatArticle>('fetch_wechat_article', { url: u })
      setArticle(art)
      setUrl(u)
      saveHistory(u, art.title)
      setHistory(loadHistory())
      message.success('抓取成功')
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setFetching(false)
    }
  }

  const handleAnalyze = async () => {
    if (!article) return
    setAnalyzing(true)
    setAnalysis('')
    try {
      const content = article.content_text.slice(0, CONTENT_LIMIT)
      const user = `文章标题：${article.title}\n公众号：${article.account || '未知'}\n作者：${article.author || '未知'}\n发布时间：${article.publish_time || '未知'}\n\n正文：\n${content}`
      const md = await tauriInvoke<string>('ai_chat', {
        systemPrompt: ANALYZE_SYSTEM,
        userPrompt: user,
      })
      setAnalysis(md)
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setAnalyzing(false)
    }
  }

  const handleWrite = async () => {
    if (!article || !analysis) return
    setWriting(true)
    setGenerated(null)
    setQuality(null)
    try {
      const user = `## 爆款拆解报告\n${analysis}\n\n## 我的选题要求\n主题/角度：${topic.trim() || '与原文同领域，换一个全新的案例和切入角度'}\n目标字数：${wordCount} 字左右\n写作风格：${style.trim() || '参考原作风格'}`
      const r = await tauriInvoke<QualityGateResult>('generate_with_quality_gate', {
        systemPrompt: WRITE_SYSTEM,
        userPrompt: user,
        knobs,
      })
      setGenerated({ title: r.title || '同款文章', content: r.content })
      setQuality({ report: r.report, score: r.score, rounds: r.rounds })
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setWriting(false)
    }
  }

  const handleSave = async () => {
    if (!generated || !article) return
    setSaving(true)
    try {
      const id = Date.now().toString()
      const summary = generated.content.replace(/[#*\n]/g, '').slice(0, 60)
      await tauriInvoke('save_article', {
        article: {
          id,
          title: generated.title,
          content: generated.content,
          summary,
          status: 'draft',
          folder_id: null,
          wechat_article_id: null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          cover_image: article.cover || undefined,
        },
      })
      message.success('已已存入内容工作台（草稿）')
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setSaving(false)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>
          爆文工坊
        </Title>
      </div>

      <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        <div style={{ maxWidth: 860, margin: '0 auto' }}>
        <Tabs
          activeKey={activeTab}
          onChange={setActiveTab}
          items={[
            {
              key: 'write',
              label: '写同款',
              children: (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>

          {/* 历史记录 */}
          {history.length > 0 && (
            <div>
              <Space wrap size={4}>
                <Text type="secondary" style={{ fontSize: 12 }}>最近抓取：</Text>
                {history.slice(0, 5).map((h) => (
                  <Button key={h.url} size="small" type="text" onClick={() => handleFetch(h.url)} style={{ fontSize: 12 }}>
                    {h.title.length > 18 ? h.title.slice(0, 18) + '…' : h.title}
                  </Button>
                ))}
              </Space>
            </div>
          )}

          {/* 步骤指示 */}
          <Steps
            size="small"
            current={step}
            items={[
              { title: '抓取爆文', icon: <FileSearchOutlined /> },
              { title: 'AI 拆解', icon: <FileSearchOutlined /> },
              { title: '写同款', icon: <EditOutlined /> },
            ]}
          />

          {/* 第一步：抓取 */}
          <Card title="1. 爆文链接">
            <Space.Compact style={{ width: '100%' }}>
              <Input
                placeholder="粘贴微信公众号文章链接（mp.weixin.qq.com）"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onPressEnter={() => handleFetch()}
              />
              <Button type="primary" loading={fetching} onClick={() => handleFetch()}>
                抓取
              </Button>
            </Space.Compact>
            {fetching && <Spin style={{ marginTop: 12 }} />}
            {article && (
              <div style={{ marginTop: 12 }}>
                <Paragraph style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>{article.title}</Paragraph>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {article.account || '未知公众号'}
                  {article.author ? ` · ${article.author}` : ''}
                  {article.publish_time ? ` · ${article.publish_time}` : ''}
                  {` · 约 ${article.content_text.length} 字`}
                </Text>
                <Paragraph
                  type="secondary"
                  style={{ fontSize: 12, marginTop: 8, marginBottom: 0 }}
                  ellipsis={{ rows: 3, expandable: true, symbol: '展开' }}
                >
                  {article.content_text.slice(0, 500)}
                </Paragraph>
              </div>
            )}
          </Card>

          {/* 第二步：拆解 */}
          <Card
            title="2. AI 拆解爆款要素"
            extra={
              article && (
                <Space>
                  {analysis && (
                    <Button
                      loading={savingPersona}
                      onClick={async () => {
                        setSavingPersona(true)
                        try {
                          const b = await tauriInvoke<{ name: string }>('create_blogger_from_analysis', { analysis })
                          message.success(`已从拆解报告沉淀人设「${b.name}」，可在「写作人设」页查看`)
                        } catch (e) {
                          message.error(getErrorMessage(e))
                        } finally {
                          setSavingPersona(false)
                        }
                      }}
                    >
                      存为写作人设
                    </Button>
                  )}
                  <Button type="primary" ghost loading={analyzing} onClick={handleAnalyze} icon={<FileSearchOutlined />}>
                    {analysis ? '重新拆解' : '开始拆解'}
                  </Button>
                </Space>
              )
            }
          >
            {!article && <Text type="secondary">先抓取一篇爆文</Text>}
            {analyzing && <Spin style={{ marginRight: 8 }} />}
            {analysis && (
              <div
                className="hot-analysis"
                dangerouslySetInnerHTML={{ __html: marked(analysis) as string }}
              />
            )}
            {!analysis && article && !analyzing && (
              <Text type="secondary">点击右上角"开始拆解"，AI 将分析标题公式、选题、结构与情绪点</Text>
            )}
          </Card>

          {/* 第三步：写同款 */}
          <Card
            title="3. 写同款"
            extra={
              analysis && (
                <Button
                  type="primary"
                  loading={writing}
                  onClick={handleWrite}
                  icon={<FireOutlined />}
                >
                  {generated ? '重新生成' : '生成同款文章'}
                </Button>
              )
            }
          >
            {!analysis && <Text type="secondary">先完成拆解</Text>}
            {analysis && (
              <>
                <Space direction="vertical" size={8} style={{ width: '100%', marginBottom: 12 }}>
                  <Space wrap>
                    <Text type="secondary" style={{ fontSize: 12 }}>微调旋钮：</Text>
                    <Select
                      allowClear
                      placeholder="开头钩子"
                      style={{ width: 140 }}
                      options={HOOK_OPTIONS}
                      value={knobs.hook || undefined}
                      onChange={(v) => setKnobs((k) => ({ ...k, hook: v }))}
                    />
                    <Select
                      allowClear
                      placeholder="情绪浓度"
                      style={{ width: 110 }}
                      options={EMOTION_OPTIONS}
                      value={knobs.emotion || undefined}
                      onChange={(v) => setKnobs((k) => ({ ...k, emotion: v }))}
                    />
                    <Space size={4}>
                      <Switch size="small" checked={knobs.colloquial} onChange={(v) => setKnobs((k) => ({ ...k, colloquial: v }))} />
                      <Text style={{ fontSize: 12 }}>口语化</Text>
                    </Space>
                    <Space size={4}>
                      <Switch size="small" checked={knobs.cta} onChange={(v) => setKnobs((k) => ({ ...k, cta: v }))} />
                      <Text style={{ fontSize: 12 }}>结尾 CTA</Text>
                    </Space>
                  </Space>
                  <Input
                    placeholder="我的选题/角度（留空则同领域换新案例）"
                    value={topic}
                    onChange={(e) => setTopic(e.target.value)}
                  />
                  <Space>
                    <InputNumber min={500} max={5000} step={100} value={wordCount} onChange={(v) => setWordCount(v || 1500)} addonBefore="字数" />
                    <Input
                      placeholder="风格（留空参考原作）"
                      value={style}
                      onChange={(e) => setStyle(e.target.value)}
                      style={{ width: 240 }}
                    />
                  </Space>
                </Space>
                {writing && <Spin style={{ marginRight: 8 }} tip="AI 正在写作…" />}
                {generated && (
                  <div>
                    <Alert
                      type="success"
                      showIcon
                      message={generated.title}
                      style={{ marginBottom: 12 }}
                      action={
                        <Space>
                          <Button size="small" icon={<CopyOutlined />} onClick={() => navigator.clipboard.writeText(generated.content).then(() => message.success('正文已复制'))}>
                            复制正文
                          </Button>
                          <Button size="small" type="primary" icon={<SaveOutlined />} loading={saving} onClick={handleSave}>
                            存入内容工作台
                          </Button>
                        </Space>
                      }
                    />
                    {quality && (
                      <Collapse
                        size="small"
                        style={{ marginBottom: 12 }}
                        items={[
                          {
                            key: 'quality',
                            label: `质量体检：${quality.score} 分（${quality.rounds > 0 ? `经 ${quality.rounds} 轮自动重写` : '一次达标'}）`,
                            children: (
                              <div
                                className="hot-analysis"
                                style={{ fontSize: 12 }}
                                dangerouslySetInnerHTML={{ __html: marked(quality.report) as string }}
                              />
                            ),
                          },
                        ]}
                      />
                    )}
                    <div
                      className="hot-analysis"
                      style={{ background: '#fafafa', borderRadius: 8, padding: 16 }}
                      dangerouslySetInnerHTML={{ __html: marked(generated.content) as string }}
                    />
                  </div>
                )}
                {!generated && !writing && <Text type="secondary">设置选题后点击"生成同款文章"</Text>}
              </>
            )}
          </Card>
        </div>
              ),
            },
            {
              key: 'radar',
              label: '热点雷达',
              children: <HotRadarPanel />,
            },
          ]}
        />
        </div>
      </div>
    </div>
  )
}
