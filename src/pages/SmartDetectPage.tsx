/**
 * 智能体检 · 原创度检测：粘贴文章，仿照朱雀等检测器的判定维度，
 * 判断内容更像“真人创作”还是“AI 生成”。
 * 注意：结果为 AI 自查参考，不接入朱雀等官方服务，也不构成通过检测的保证。
 */
import { useState } from 'react'
import {
  Alert, Button, Card, Input, Popover, Select, Space, Tag, Typography, message,
} from 'antd'
import {
  ClearOutlined, HighlightOutlined, QuestionCircleOutlined, SearchOutlined,
} from '@ant-design/icons'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { getErrorMessage } from '../utils/getErrorMessage'

const { Title, Text, Paragraph } = Typography

const MIN_CHARS = 50
const MAX_CHARS = 20000
const DISCLAIMER_HIDDEN_KEY = 'smart-check-detect-disclaimer-hidden'

export interface DetectionReport {
  verdict: string
  ai_probability: number
  confidence: number
  llm_verdict: string
  human_fingerprint: string
  llm_ai_probability: number
  local_ai_probability: number
  local_signals: string[]
  feature_breakdown?: {
    ai_like: number
    human_like: number
    suspected: number
  }
  summary: string
  reasons: string[]
  suggestions: string[]
}

const TEXT_TYPE_OPTIONS = [
  { value: '自媒体文章（公众号等）', label: '自媒体文章（公众号等）' },
  { value: '百家号/头条资讯', label: '百家号/头条资讯' },
  { value: '小红书/短内容', label: '小红书/短内容' },
  { value: '小说/故事/网文', label: '小说/故事/网文' },
  { value: '通用文本/其他', label: '通用文本/其他' },
]

export const VERDICT_META: Record<string, { label: string; color: string }> = {
  likely_ai: { label: '疑似 AI 创作', color: 'red' },
  mixed: { label: '人机混合 / 难以判断', color: 'orange' },
  likely_human: { label: '疑似人工创作', color: 'green' },
  uncertain: { label: '特征不足，无法判断', color: 'default' },
}

export const FINGERPRINT_META: Record<string, { label: string; color: string }> = {
  strong: { label: '强人工指纹', color: 'green' },
  weak: { label: '人工指纹弱', color: 'orange' },
  none: { label: '无人工指纹', color: 'red' },
}

interface DonutSegment {
  key: 'ai_like' | 'suspected' | 'human_like'
  label: string
  percent: number
  color: string
}

export function FeatureDonut({ breakdown }: { breakdown: NonNullable<DetectionReport['feature_breakdown']> }) {
  const segments: DonutSegment[] = [
    { key: 'ai_like', label: '符合 AI 特征', percent: breakdown.ai_like, color: '#f5222d' },
    { key: 'suspected', label: '疑似 AI / 拿不准', percent: breakdown.suspected, color: '#fa8c16' },
    { key: 'human_like', label: '符合人工特征', percent: breakdown.human_like, color: '#52c41a' },
  ]
  const center = [...segments].sort((a, b) => b.percent - a.percent)[0]
  const CX = 250
  const CY = 150
  const R = 68
  const STROKE = 25
  const ringOuter = R + STROKE / 2
  const labelRadius = ringOuter + 24
  let accDeg = 0

  return (
    <svg
      width={430}
      height={250}
      viewBox="0 0 500 300"
      style={{ flexShrink: 0, maxWidth: '100%' }}
    >
      <circle cx={CX} cy={CY} r={R} fill="none" stroke="#f0f0f0" strokeWidth={STROKE} />
      {segments.filter((s) => s.percent > 0).map((s) => {
        const startDeg = accDeg - 90
        const centerDeg = accDeg + (s.percent / 100) * 360 / 2 - 90
        const rad = (centerDeg * Math.PI) / 180
        const cos = Math.cos(rad)
        const sin = Math.sin(rad)
        const lineEndX = CX + cos * labelRadius
        const lineEndY = CY + sin * labelRadius
        const anchor = Math.abs(cos) < 0.25
          ? 'middle'
          : cos >= 0
            ? 'start'
            : 'end'
        const textX = anchor === 'start'
          ? lineEndX + 8
          : anchor === 'end'
            ? lineEndX - 8
            : lineEndX
        const textY = lineEndY + 4
        accDeg += (s.percent / 100) * 360
        return (
          <g key={s.key}>
            <circle
              cx={CX}
              cy={CY}
              r={R}
              fill="none"
              stroke={s.color}
              strokeWidth={STROKE}
              pathLength={100}
              strokeDasharray={`${s.percent} ${100 - s.percent}`}
              transform={`rotate(${startDeg} ${CX} ${CY})`}
            />
            <line
              x1={CX + cos * (ringOuter + 3)}
              y1={CY + sin * (ringOuter + 3)}
              x2={lineEndX}
              y2={lineEndY}
              stroke={s.color}
              strokeWidth={1.4}
            />
            <text
              x={textX}
              y={textY}
              textAnchor={anchor}
              fontSize={12}
              fill="#595959"
            >
              {s.label} {s.percent}%
            </text>
          </g>
        )
      })}
      <text
        x={CX}
        y={CY - 6}
        textAnchor="middle"
        fontSize={12}
        fill="#8c8c8c"
      >
        主要占比
      </text>
      <text
        x={CX}
        y={CY + 14}
        textAnchor="middle"
        fontSize={14}
        fontWeight={600}
        fill={center.color}
      >
        {center.label} {center.percent}%
      </text>
    </svg>
  )
}

export default function SmartDetectPage() {
  const navigate = useNavigate()
  const [content, setContent] = useState('')
  const [textType, setTextType] = useState<string>(TEXT_TYPE_OPTIONS[0].value)
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<DetectionReport | null>(null)
  const [disclaimerVisible, setDisclaimerVisible] = useState(
    () => localStorage.getItem(DISCLAIMER_HIDDEN_KEY) !== '1',
  )

  const handleDetect = async () => {
    const text = content.trim()
    if (text.length < MIN_CHARS) {
      message.warning(`内容太短（至少 ${MIN_CHARS} 字），特征不足难以有效检测`)
      return
    }
    if (text.length > MAX_CHARS) {
      message.warning(`内容过长（最多 ${MAX_CHARS} 字），请分段检测`)
      return
    }
    setLoading(true)
    setResult(null)
    try {
      const report = await tauriInvoke<DetectionReport>('detect_ai_authorship', {
        content: text,
        textType,
      })
      setResult(report)
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setLoading(false)
    }
  }

  const verdictMeta = result ? VERDICT_META[result.verdict] : null
  const hideDisclaimer = () => {
    localStorage.setItem(DISCLAIMER_HIDDEN_KEY, '1')
    setDisclaimerVisible(false)
  }

  const goDeAiLoop = () => {
    navigate('/smart-check/humanize', {
      state: { content, textType, autoRun: true, beforeReport: result },
    })
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Title level={4} style={{ margin: 0 }}>原创度检测</Title>
          <Popover
            trigger="click"
            content={
              <div style={{ maxWidth: 520 }}>
                <Text style={{ fontSize: 13 }}>
                  仿照朱雀等 AI 检测工具的常见判定维度（模板句式、结构套路、具体细节、个人痕迹、语言节奏），
                  并先用多类中文 AI 腔规则（空泛升华、套话开头结尾、AI 黑话、同义堆叠、排比三连、
                  翻案腔、感悟式升华、聊天残渣等）加统计特征做本地扫描，再用大模型对整篇文本做创作归属评估。它是写作自查参考，
                  不接入朱雀官方接口，也不保证与任何
                  第三方检测结果一致；“AI 味低”也不等于查重意义上的“原创”。
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
              onClose={hideDisclaimer}
              message="检测口径说明"
              description="检测的是“文本像不像 AI 写的”，不是查重/抄袭检测。不同检测平台算法会持续更新、也存在误判，任何工具都不能保证检测结果或“通过某平台检测”。"
            />
          )}

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'minmax(0, 3fr) minmax(0, 2fr)',
              gap: 16,
              alignItems: 'stretch',
            }}
          >
            <Card
              title="待检测文章"
              style={{ minHeight: 600 }}
              extra={
              <Space>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {content.length} / {MAX_CHARS}
                </Text>
                <Button
                  icon={<ClearOutlined />}
                  disabled={loading}
                  onClick={() => { setContent(''); setResult(null) }}
                >
                  清空
                </Button>
                <Button
                  type="primary"
                  icon={<SearchOutlined />}
                  loading={loading}
                  onClick={() => void handleDetect()}
                >
                  开始检测
                </Button>
              </Space>
            }
          >
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <Space wrap>
                <Text style={{ fontSize: 13 }}>文本类型：</Text>
                <Select
                  style={{ width: 220 }}
                  value={textType}
                  onChange={setTextType}
                  options={TEXT_TYPE_OPTIONS}
                />
              </Space>
              <Input.TextArea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                placeholder={`粘贴要检测的文章全文（${MIN_CHARS}~${MAX_CHARS} 字）。\n支持公众号、头条、小红书、小说等文体，选对文本类型结果更准。`}
                autoSize={{ minRows: 24, maxRows: 38 }}
                disabled={loading}
              />
              <Text type="secondary" style={{ fontSize: 12 }}>
                检测需要调用已配置的 DeepSeek API，通常 20~60 秒；请勿包含需要保密的个人敏感信息。
              </Text>
            </div>
          </Card>

          {!loading && !result && (
            <Card
              title="检测结果（参考）"
              style={{ height: '100%' }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  minHeight: 600,
                  textAlign: 'center',
                  color: '#8c8c8c',
                }}
              >
                <Text style={{ fontSize: 14 }}>
                  点击左侧“开始检测”，结果会显示在这里
                </Text>
              </div>
            </Card>
          )}

          {loading && (
            <Card>
              <div style={{ textAlign: 'center', padding: '24px 0' }}>
                <SearchOutlined style={{ fontSize: 28, color: '#1677ff' }} />
                <Paragraph style={{ marginTop: 12, marginBottom: 0 }}>
                  AI 正在从句式、结构、细节与个人痕迹等维度分析全文…
                </Paragraph>
              </div>
            </Card>
          )}

          {result && verdictMeta && (
            <Card
              title="检测结果（参考）"
              extra={
                <Space>
                  <Button
                    icon={<HighlightOutlined />}
                    size="small"
                    type="primary"
                    ghost
                    onClick={goDeAiLoop}
                  >
                    去 AI 味儿复检
                  </Button>
                </Space>
              }
            >
              <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 8 }}>
                  <Space size={8}>
                    <Tag color={verdictMeta.color} style={{ fontSize: 13, padding: '2px 10px' }}>
                      {verdictMeta.label}
                    </Tag>
                    <Tag
                      color={FINGERPRINT_META[result.human_fingerprint]?.color ?? 'default'}
                      style={{ fontSize: 12, padding: '2px 8px' }}
                    >
                      {FINGERPRINT_META[result.human_fingerprint]?.label ?? result.human_fingerprint}
                    </Tag>
                  </Space>
                </div>

                {result.feature_breakdown && (
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      flexWrap: 'wrap',
                      gap: 24,
                      padding: '16px 8px 8px',
                      borderTop: '1px solid #f0f0f0',
                    }}
                  >
                    <FeatureDonut breakdown={result.feature_breakdown} />
                  </div>
                )}

                <Paragraph style={{ marginBottom: 0, background: '#fafafa', padding: 12, borderRadius: 8 }}>
                  <Text strong>结论：</Text>
                  <br />
                  {result.summary}
                </Paragraph>

                {result.reasons.length > 0 && (
                  <div>
                    <Text strong style={{ fontSize: 13 }}>判断依据</Text>
                    <ol style={{ marginTop: 8, marginBottom: 0, paddingLeft: 20 }}>
                      {result.reasons.map((reason, idx) => (
                        <li key={idx} style={{ marginBottom: 6, lineHeight: 1.7 }}>{reason}</li>
                      ))}
                    </ol>
                  </div>
                )}

                {result.suggestions.length > 0 && (
                  <Alert
                    type="warning"
                    showIcon
                    message="降低 AI 味建议"
                    description={
                      <ol style={{ margin: '8px 0 0', paddingLeft: 20 }}>
                        {result.suggestions.map((s, idx) => (
                          <li key={idx} style={{ marginBottom: 4 }}>{s}</li>
                        ))}
                      </ol>
                    }
                  />
                )}
              </div>
            </Card>
          )}

          </div>

        </div>
      </div>
    </div>
  )
}
