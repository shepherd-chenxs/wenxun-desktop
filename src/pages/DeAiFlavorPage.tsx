/**
 * 智能体检 · 一键去 AI 味儿：对整篇初稿做“去模板化”改写，
 * 保留事实与观点，降低机械排比、万能金句等 AI 高频特征。
 */
import { useEffect, useState } from 'react'
import {
  Alert, Button, Card, Input, Popover, Progress, Select, Space, Tag, Typography, message,
} from 'antd'
import {
  CopyOutlined, HighlightOutlined, QuestionCircleOutlined, ReloadOutlined,
} from '@ant-design/icons'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { useLocation } from 'react-router-dom'
import {
  FeatureDonut, FINGERPRINT_META, VERDICT_META, type DetectionReport,
} from './SmartDetectPage'
import { getErrorMessage } from '../utils/getErrorMessage'

const { Title, Text, Paragraph } = Typography

const MIN_CHARS = 50
const MAX_CHARS = 20000
const DISCLAIMER_HIDDEN_KEY = 'smart-check-humanize-disclaimer-hidden'

const STYLE_OPTIONS = [
  { value: '公众号文章：自然、口语但有信息密度', label: '公众号文章' },
  { value: '小红书/短内容：轻松、具体、有画面感', label: '小红书/短内容' },
  { value: '资讯/头条：信息清楚但不八股', label: '资讯/头条' },
  { value: '通用自然表达：保留作者原有语气', label: '通用自然表达' },
]

function textTypeFromStyle(style: string): string {
  if (style.includes('小红书')) return '小红书/短内容'
  if (style.includes('资讯') || style.includes('头条')) return '百家号/头条资讯'
  if (style.includes('通用')) return '通用文本/其他'
  return '自媒体文章（公众号等）'
}

function styleFromTextType(textType?: string): string {
  if (textType?.includes('小红书')) return STYLE_OPTIONS[1].value
  if (textType?.includes('资讯') || textType?.includes('头条')) return STYLE_OPTIONS[2].value
  if (textType?.includes('通用')) return STYLE_OPTIONS[3].value
  return STYLE_OPTIONS[0].value
}

function aiColor(ai: number): string {
  if (ai >= 70) return '#f5222d'
  if (ai >= 45) return '#fa8c16'
  return '#52c41a'
}

export default function DeAiFlavorPage() {
  const location = useLocation()
  const routeState = location.state as {
    content?: string
    textType?: string
    autoRun?: boolean
    beforeReport?: DetectionReport
  } | null
  const [content, setContent] = useState(routeState?.content ?? '')
  const [style, setStyle] = useState<string>(() => styleFromTextType(routeState?.textType))
  const [loading, setLoading] = useState(false)
  const [rewritten, setRewritten] = useState('')
  const [checkReport, setCheckReport] = useState<DetectionReport | null>(null)
  const [originalReport] = useState<DetectionReport | null>(routeState?.beforeReport ?? null)
  const [disclaimerVisible, setDisclaimerVisible] = useState(
    () => localStorage.getItem(DISCLAIMER_HIDDEN_KEY) !== '1',
  )

  const handleHumanize = async () => {
    const text = content.trim()
    if (text.length < MIN_CHARS) {
      message.warning(`内容太短（至少 ${MIN_CHARS} 字），请粘贴完整稿件`)
      return
    }
    if (text.length > MAX_CHARS) {
      message.warning(`内容过长（最多 ${MAX_CHARS} 字），请分段处理`)
      return
    }
    setLoading(true)
    setRewritten('')
    setCheckReport(null)
    try {
      const result = await tauriInvoke<{ text: string; report: DetectionReport }>('humanize_ai_text_with_check', {
        content: text,
        style,
        textType: textTypeFromStyle(style),
      })
      setRewritten(result.text)
      setCheckReport(result.report)
      message.success('去 AI 味完成，已自动复检')
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setLoading(false)
    }
  }

  const copyResult = async () => {
    if (!rewritten) return
    try {
      await navigator.clipboard.writeText(rewritten)
      message.success('改写结果已复制')
    } catch {
      message.error('复制失败，请手动选择文本复制')
    }
  }

  const useAsNewOriginal = () => {
    if (!rewritten) return
    setContent(rewritten)
    setRewritten('')
    setCheckReport(null)
    message.success('已把改写结果放回输入框，可继续去味或重新检测')
  }

  const checkMeta = checkReport ? VERDICT_META[checkReport.verdict] : null

  useEffect(() => {
    if (routeState?.autoRun && routeState.content) {
      void handleHumanize()
    }
    // 只在该页面刚被“原创度检测”带过来时自动开始一次
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Title level={4} style={{ margin: 0 }}>一键去 AI 味儿</Title>
          <Popover
            trigger="click"
            content={
              <div style={{ maxWidth: 520 }}>
                <Text style={{ fontSize: 13 }}>
                  本地规则会先自动清理可机械替换的 AI 腔（赋能→支持、抓手→切入点，删除“众所周知/
                  综上所述/希望对你有所帮助”等套话与聊天残渣，并限流破折号、感叹号），再针对机械排比、
                  翻案腔（不是A而是B、不是A也不是B是因为C）、感悟式升华（你突然明白了/真正的爱是/
                  就像我们的爱情）、万能金句、过度总分总、线索过度收束等判断级特征做整篇改写，
                  目标是让文章更像真人一笔一笔写出来。改写保留事实与结构，不会为了“过检测”而
                  插入错字或乱码；发布前请人工复核数据与表述。
                </Text>
              </div>
            }
          >
            <QuestionCircleOutlined style={{ marginLeft: 8, fontSize: 15, color: '#8c8c8c', cursor: 'pointer' }} />
          </Popover>
        </div>
      </div>

      <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', display: 'flex', flexDirection: 'column', gap: 16 }}>
          {disclaimerVisible && (
            <Alert
              type="info"
              showIcon
              closable
              onClose={() => {
                localStorage.setItem(DISCLAIMER_HIDDEN_KEY, '1')
                setDisclaimerVisible(false)
              }}
              message="改写说明"
              description="请只粘贴你自己有权限修改的稿件。改写不会新增事实，但仍建议发布前逐句复核；软件不保证任何第三方 AI 检测结果，请勿对外作“包过检测”承诺。"
            />
          )}

          <Card
            title="原文 ⇆ 改写结果"
            extra={
              <Button
                type="primary"
                icon={<HighlightOutlined />}
                loading={loading}
                onClick={() => void handleHumanize()}
              >
                一键去 AI 味儿
              </Button>
            }
          >
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'minmax(0, 1fr) 1px minmax(0, 1fr)',
                }}
              >
                <div style={{ paddingRight: 20, minWidth: 0 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 10 }}>
                    <Text strong style={{ fontSize: 15 }}>原文</Text>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {content.length} 字
                    </Text>
                    <Space style={{ marginLeft: 'auto' }}>
                      <Select
                        size="small"
                        style={{ width: 180 }}
                        value={style}
                        onChange={setStyle}
                        options={STYLE_OPTIONS}
                      />
                      <Button
                        size="small"
                        icon={<ReloadOutlined />}
                        onClick={() => { setContent(''); setRewritten(''); setCheckReport(null) }}
                      >
                        清空
                      </Button>
                    </Space>
                  </div>
                  <Input.TextArea
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                    placeholder={`粘贴 AI 味偏重的初稿（${MIN_CHARS}~${MAX_CHARS} 字）`}
                    style={{ height: 520, minHeight: 520, resize: 'none' }}
                    disabled={loading}
                  />
                </div>

                <div style={{ width: 1, background: '#f0f0f0' }} />

                <div style={{ paddingLeft: 20, minWidth: 0 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 10 }}>
                    <Text strong style={{ fontSize: 15 }}>改写结果</Text>
                    {rewritten && (
                      <Text type="secondary" style={{ fontSize: 12 }}>
                        约 {rewritten.length} 字
                      </Text>
                    )}
                    <Space style={{ marginLeft: 'auto' }}>
                      {rewritten && (
                        <>
                          <Button size="small" icon={<CopyOutlined />} onClick={() => void copyResult()}>
                            复制全文
                          </Button>
                          <Button size="small" type="primary" ghost onClick={useAsNewOriginal}>
                            放回原文继续改
                          </Button>
                        </>
                      )}
                    </Space>
                  </div>
                  {rewritten ? (
                    <div
                      style={{
                        whiteSpace: 'pre-wrap',
                        lineHeight: 1.8,
                        fontSize: 14,
                        height: 520,
                        overflow: 'auto',
                        background: '#fafafa',
                        border: '1px solid #f0f0f0',
                        borderRadius: 8,
                        padding: '12px 16px',
                      }}
                    >
                      {rewritten}
                    </div>
                  ) : (
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        height: 520,
                        background: '#fafafa',
                        border: '1px dashed #d9d9d9',
                        borderRadius: 8,
                        padding: 24,
                        textAlign: 'center',
                      }}
                    >
                      <Text type="secondary" style={{ fontSize: 14 }}>
                        {loading
                          ? 'AI 正在去味并自动复检，完成后会显示在这里…'
                          : '点击“一键去 AI 味儿”后，改写结果会显示在这里，可逐段和原文对比。'}
                      </Text>
                    </div>
                  )}
                </div>
              </div>
          </Card>

          {checkReport && checkMeta && (
            <Card
              title="去味后自动复检"
              extra={
                <Space size={8}>
                  <Tag color={checkMeta.color} style={{ fontSize: 13, padding: '2px 10px' }}>
                    {checkMeta.label}
                  </Tag>
                  <Tag
                    color={FINGERPRINT_META[checkReport.human_fingerprint]?.color ?? 'default'}
                    style={{ fontSize: 12, padding: '2px 8px' }}
                  >
                    {FINGERPRINT_META[checkReport.human_fingerprint]?.label ?? checkReport.human_fingerprint}
                  </Tag>
                </Space>
              }
            >
              <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
                <div style={{ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: 24 }}>
                  {checkReport.feature_breakdown && (
                    <FeatureDonut breakdown={checkReport.feature_breakdown} />
                  )}
                  <div style={{ minWidth: 240, flex: 1 }}>
                    <Text style={{ fontSize: 13 }}>复检 AI 特征倾向</Text>
                    <Progress
                      percent={checkReport.ai_probability}
                      strokeColor={aiColor(checkReport.ai_probability)}
                      format={(p) => `${p ?? 0}%`}
                    />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      综合 {checkReport.ai_probability}% = 模型主观 {checkReport.llm_ai_probability}% × 60% +
                      规则扫描 {checkReport.local_ai_probability}% × 40%
                    </Text>
                    <Paragraph style={{ marginTop: 10, marginBottom: 0 }}>
                      {checkReport.summary}
                    </Paragraph>
                  </div>
                </div>

                {originalReport && (
                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(3, 1fr)',
                      gap: 12,
                      background: '#fafafa',
                      borderRadius: 8,
                      padding: '10px 14px',
                    }}
                  >
                    <div>
                      <Text type="secondary" style={{ fontSize: 12 }}>AI 特征倾向</Text>
                      <br />
                      <Text strong style={{ fontSize: 18, color: originalReport.ai_probability > checkReport.ai_probability ? '#52c41a' : '#f5222d' }}>
                        {originalReport.ai_probability}% → {checkReport.ai_probability}%
                      </Text>
                    </div>
                    <div>
                      <Text type="secondary" style={{ fontSize: 12 }}>AI 特征占比</Text>
                      <br />
                      <Text strong style={{ fontSize: 18 }}>
                        {originalReport.feature_breakdown?.ai_like ?? '-'}% → {checkReport.feature_breakdown?.ai_like ?? '-'}%
                      </Text>
                    </div>
                    <div>
                      <Text type="secondary" style={{ fontSize: 12 }}>人工特征占比</Text>
                      <br />
                      <Text strong style={{ fontSize: 18, color: (checkReport.feature_breakdown?.human_like ?? 0) > (originalReport.feature_breakdown?.human_like ?? 0) ? '#52c41a' : undefined }}>
                        {originalReport.feature_breakdown?.human_like ?? '-'}% → {checkReport.feature_breakdown?.human_like ?? '-'}%
                      </Text>
                    </div>
                  </div>
                )}

                {checkReport.reasons.length > 0 && (
                  <div>
                    <Text strong style={{ fontSize: 13 }}>复检依据</Text>
                    <ol style={{ marginTop: 8, marginBottom: 0, paddingLeft: 20 }}>
                      {checkReport.reasons.map((reason, idx) => (
                        <li key={idx} style={{ marginBottom: 6, lineHeight: 1.7 }}>{reason}</li>
                      ))}
                    </ol>
                  </div>
                )}

                <Alert
                  type={checkReport.verdict === 'likely_ai' ? 'warning' : 'info'}
                  showIcon
                  message={
                    checkReport.verdict === 'likely_ai'
                      ? '复检仍偏高：可点“放回原文继续改”再跑一轮，或到原创度检测页对比前后报告'
                      : '复检完成：可点“复制全文”，或放回原文继续微调'
                  }
                />
              </div>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
