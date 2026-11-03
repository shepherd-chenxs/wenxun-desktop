/**
 * 质检库：质量体检报告存档、筛选、勾选多篇后生成写手提升建议。
 * 自定义写手支持逐字段应用修订；内置写手与【当前用户】仅提供建议。
 */
import React, { useEffect, useMemo, useState } from 'react';
import {
  Alert, Button, DatePicker, Drawer, Empty, Input, message, Modal, Popover, Popconfirm, Select, Space, Table, Tag, Typography,
} from 'antd';
import { DeleteOutlined, QuestionCircleOutlined, ReloadOutlined, ThunderboltOutlined } from '@ant-design/icons';
import dayjs, { type Dayjs } from 'dayjs';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { storeService, type VirtualBlogger } from '../services/storeService';
import QualityReportView from '../components/editor/QualityReportView';

const { Title, Text, Paragraph } = Typography;
const { RangePicker } = DatePicker;

const CURRENT_USER_LABEL = '【当前用户】'

interface QualityReportRow {
  id: string
  article_id: string | null
  article_title: string
  source: string
  persona_name: string | null
  score: number
  report: string
  created_at: string
}

interface PersonaImprovement {
  summary: string
  suggestions: string[]
  updates: Record<string, string>
  report_count: number
  can_apply: boolean
  blogger_id: string | null
  blogger_name: string | null
}

const FIELD_META: Record<string, string> = {
  writing_style: '选题与表达要领',
  language_style: '语言风格',
  article_structure: '文章结构',
  layout_format: '排版规范',
  tone_keywords: '语气关键词',
  content_bottom_line: '内容红线',
}


const QualityLibPage: React.FC = () => {
  const [reports, setReports] = useState<QualityReportRow[]>([])
  const [loading, setLoading] = useState(false)
  const [preview, setPreview] = useState<QualityReportRow | null>(null)

  // 筛选
  const [writerFilter, setWriterFilter] = useState<string | undefined>()
  const [dateRange, setDateRange] = useState<[Dayjs | null, Dayjs | null] | null>(null)
  const [keyword, setKeyword] = useState('')

  // 综合分析
  const [bloggers, setBloggers] = useState<VirtualBlogger[]>([])
  const [analysisTarget, setAnalysisTarget] = useState<VirtualBlogger | null>(null)
  const [analyzing, setAnalyzing] = useState(false)
  const [analysis, setAnalysis] = useState<PersonaImprovement | null>(null)
  const [analysisOpen, setAnalysisOpen] = useState(false)
  const [checked, setChecked] = useState<Record<string, boolean>>({})
  const [applying, setApplying] = useState(false)

  // 多选
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([])

  const loadReports = async () => {
    setLoading(true)
    try {
      setReports(await tauriInvoke<QualityReportRow[]>('get_quality_report_list', { personaName: null }))
      setSelectedRowKeys([])
    } catch (e) {
      message.error((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void loadReports()
    storeService.getVirtualBloggers().then(setBloggers).catch(() => {})
  }, [])

  useEffect(() => {
    setSelectedRowKeys([])
  }, [writerFilter, keyword, dateRange?.[0]?.valueOf(), dateRange?.[1]?.valueOf()])

  const writerOptions = useMemo(() => {
    const labels = new Set<string>([CURRENT_USER_LABEL])
    bloggers.forEach((b) => { if (b.name.trim()) labels.add(b.name.trim()) })
    reports.forEach((r) => labels.add(r.persona_name?.trim() || CURRENT_USER_LABEL))
    return Array.from(labels)
      .sort((a, b) => a.localeCompare(b, 'zh-Hans-CN'))
      .map((label) => ({ value: label, label }))
  }, [bloggers, reports])

  const filteredReports = useMemo(() => {
    const kw = keyword.trim().toLowerCase()
    const start = dateRange?.[0]?.startOf('day')
    const end = dateRange?.[1]?.endOf('day')
    return reports.filter((r) => {
      const writer = r.persona_name?.trim() || CURRENT_USER_LABEL
      if (writerFilter && writer !== writerFilter) return false
      if (kw && !r.article_title.toLowerCase().includes(kw)) return false
      const t = dayjs(r.created_at)
      if (start && t.isBefore(start)) return false
      if (end && t.isAfter(end)) return false
      return true
    })
  }, [reports, writerFilter, dateRange, keyword])

  const selectedReports = useMemo(
    () => filteredReports.filter((r) => selectedRowKeys.includes(r.id)),
    [filteredReports, selectedRowKeys],
  )
  const selectedWriterCount = new Set(
    selectedReports.map((r) => r.persona_name?.trim() || CURRENT_USER_LABEL),
  ).size

  const handleAnalyze = async () => {
    if (selectedReports.length < 2) {
      message.warning('请至少勾选 2 篇质检报告后再生成提升建议')
      return
    }
    if (selectedWriterCount > 1) {
      message.warning('勾选的报告来自多个写手，请先筛选到同一写手（或同一批【当前用户】记录）')
      return
    }

    setAnalyzing(true)
    setAnalysis(null)
    setAnalysisOpen(true)
    try {
      const r = await tauriInvoke<PersonaImprovement>('analyze_selected_quality_reports', {
        reportIds: selectedReports.map((x) => x.id),
      })
      setAnalysis(r)
      const target = r.blogger_id ? bloggers.find((b) => b.id === r.blogger_id) ?? null : null
      setAnalysisTarget(target)
      const init: Record<string, boolean> = {}
      Object.keys(r.updates).forEach((k) => { init[k] = true })
      setChecked(init)
    } catch (e) {
      message.error((e as Error).message)
      setAnalysisOpen(false)
    } finally {
      setAnalyzing(false)
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await tauriInvoke('delete_quality_report', { id })
      setReports((prev) => prev.filter((r) => r.id !== id))
      setSelectedRowKeys((keys) => keys.filter((k) => k !== id))
      if (preview?.id === id) setPreview(null)
      message.success('质检报告已删除')
    } catch (e) {
      message.error((e as Error).message)
    }
  }

  const handleApply = async () => {
    if (!analysisTarget || !analysis) return
    const fields = Object.keys(analysis.updates).filter((k) => checked[k])
    if (fields.length === 0) {
      message.warning('请至少勾选一个要应用的字段')
      return
    }
    setApplying(true)
    try {
      const updated: VirtualBlogger = { ...analysisTarget }
      for (const f of fields) {
        ;(updated as any)[f] = analysis.updates[f]
      }
      await storeService.saveVirtualBlogger(updated)
      message.success(`已应用 ${fields.length} 个字段到「${analysisTarget.name}」，可在写作人设页查看`)
      setAnalysisOpen(false)
      setAnalysis(null)
    } catch (e) {
      message.error((e as Error).message)
    } finally {
      setApplying(false)
    }
  }

  const analysisTitleName = analysis?.blogger_name || CURRENT_USER_LABEL

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Title level={4} style={{ margin: 0 }}>质检库</Title>
          <Popover
            trigger="click"
            content={
              <div style={{ maxWidth: 480 }}>
                <Text style={{ fontSize: 13 }}>
                  按写手/时间/文章名筛选后，勾选同一写手的多篇报告生成提升建议。自定义写手可逐字段应用修订；内置写手与【当前用户】仅提供建议。
                </Text>
              </div>
            }
          >
            <QuestionCircleOutlined
              style={{ marginLeft: 8, fontSize: 15, color: '#8c8c8c', cursor: 'pointer' }}
            />
          </Popover>
        </div>
      </div>
      <div style={{ flexShrink: 0, height: 16, background: '#f0f2f5' }} />

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', background: '#fff', borderRadius: 8, margin: '0 16px 16px', padding: 16 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12, flexWrap: 'wrap', gap: 8 }}>
          <Space wrap>
            <Select
              allowClear
              showSearch
              placeholder="按写手搜索"
              style={{ width: 200 }}
              value={writerFilter}
              onChange={setWriterFilter}
              options={writerOptions}
            />
            <RangePicker
              value={dateRange}
              onChange={(v) => setDateRange(v)}
              style={{ maxWidth: 280 }}
            />
            <Input
              allowClear
              placeholder="按文章名搜索"
              value={keyword}
              onChange={(e) => setKeyword(e.target.value)}
              style={{ width: 220 }}
            />
            <Button icon={<ReloadOutlined />} onClick={() => void loadReports()}>刷新</Button>
          </Space>
          <Space>
            {selectedReports.length > 0 && (
              <Text type="secondary" style={{ fontSize: 12 }}>
                已选 {selectedReports.length} 篇
              </Text>
            )}
            <Button
              type="primary"
              icon={<ThunderboltOutlined />}
              loading={analyzing}
              disabled={selectedReports.length < 2}
              onClick={() => void handleAnalyze()}
            >
              生成提升建议
            </Button>
          </Space>
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          <Table
            size="small"
            rowSelection={{
              selectedRowKeys,
              onChange: (keys) => setSelectedRowKeys(keys),
            }}
            columns={[
              { title: '时间', dataIndex: 'created_at', key: 'created_at', width: 160 },
              {
                title: '文章', dataIndex: 'article_title', key: 'article_title', ellipsis: true,
                render: (t: string, r: QualityReportRow) => (
                  <a onClick={() => setPreview(r)}>{t || '（未记录）'}</a>
                ),
              },
              {
                title: '写手', dataIndex: 'persona_name', key: 'persona_name', width: 180, ellipsis: true,
                render: (p: string | null) => p?.trim() || CURRENT_USER_LABEL,
              },
              {
                title: '总分', dataIndex: 'score', key: 'score', width: 80,
                sorter: (a: QualityReportRow, b: QualityReportRow) => a.score - b.score,
                render: (s: number) => (
                  <Text type={s >= 85 ? 'success' : s >= 75 ? undefined : 'danger'} style={{ fontWeight: 600 }}>{s}</Text>
                ),
              },
              {
                title: '操作', key: 'actions', width: 90,
                render: (_: unknown, r: QualityReportRow) => (
                  <Popconfirm
                    title="确认删除这条质检报告？"
                    okText="删除"
                    okButtonProps={{ danger: true }}
                    cancelText="取消"
                    onConfirm={() => void handleDelete(r.id)}
                  >
                    <Button type="link" danger size="small" icon={<DeleteOutlined />}>
                      删除
                    </Button>
                  </Popconfirm>
                ),
              },
            ]}
            dataSource={filteredReports}
            rowKey="id"
            loading={loading}
            pagination={{ pageSize: 20, showSizeChanger: false }}
            locale={{ emptyText: reports.length === 0
              ? '质检库还是空的——去热点雷达或爆文工坊生成一篇文章，体检报告会自动归档到这里'
              : '没有符合筛选条件的质检报告' }}
          />
        </div>
      </div>

      {/* 报告预览 */}
      <Drawer
        title={preview ? `${preview.article_title || '（未记录）'} · ${preview.score} 分` : ''}
        placement="right"
        width={560}
        open={!!preview}
        onClose={() => setPreview(null)}
      >
        {preview && (
          <>
            <Space style={{ marginBottom: 12 }}>
              <Tag>{preview.persona_name?.trim() || CURRENT_USER_LABEL}</Tag>
              <Text type="secondary" style={{ fontSize: 12 }}>{preview.created_at}</Text>
            </Space>
            <QualityReportView report={preview.report} />
          </>
        )}
      </Drawer>

      {/* 综合分析与提升建议 */}
      <Modal
        title={`「${analysisTitleName}」综合分析（基于 ${analysis?.report_count ?? 0} 份质检报告）`}
        open={analysisOpen}
        footer={null}
        onCancel={() => { setAnalysisOpen(false); setAnalysis(null) }}
        width={900}
      >
        {analyzing ? (
          <div style={{ textAlign: 'center', padding: 40 }}>AI 正在聚类分析报告，通常需要 30-90 秒…</div>
        ) : analysis ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
              <Text strong>弱项模式</Text>
              <Paragraph style={{ marginBottom: 0, whiteSpace: 'pre-wrap' }}>{analysis.summary}</Paragraph>
            </div>
            {analysis.suggestions.length > 0 && (
              <div>
                <Text strong>提升建议</Text>
                <ul style={{ margin: '6px 0 0', paddingLeft: 20 }}>
                  {analysis.suggestions.map((s, i) => (<li key={i} style={{ marginBottom: 4 }}>{s}</li>))}
                </ul>
              </div>
            )}

            {analysis.can_apply ? (
              Object.keys(analysis.updates).length > 0 ? (
                <div>
                  <Text strong>Prompt 修订稿（勾选后应用，应用前可在写作人设页对比）</Text>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 10, marginTop: 8 }}>
                    {Object.entries(analysis.updates).map(([field, value]) => (
                      <div key={field} style={{ border: '1px solid #eee', borderRadius: 6, padding: 10 }}>
                        <Space style={{ marginBottom: 6 }}>
                          <input
                            type="checkbox"
                            checked={!!checked[field]}
                            onChange={(e) => setChecked((c) => ({ ...c, [field]: e.target.checked }))}
                          />
                          <Text strong>{FIELD_META[field] || field}</Text>
                        </Space>
                        <div style={{ display: 'flex', gap: 8, maxHeight: 180 }}>
                          <div style={{ flex: 1, overflow: 'auto', background: '#fafafa', borderRadius: 6, padding: 8, fontSize: 12, whiteSpace: 'pre-wrap' }}>
                            <Text type="secondary">当前：</Text>
                            {(analysisTarget as any)?.[field] || '（空）'}
                          </div>
                          <div style={{ flex: 1, overflow: 'auto', background: '#f6ffed', borderRadius: 6, padding: 8, fontSize: 12, whiteSpace: 'pre-wrap' }}>
                            <Text type="secondary">建议：</Text>
                            {value}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                  <Button type="primary" style={{ marginTop: 12 }} loading={applying} onClick={() => void handleApply()}>
                    应用勾选字段
                  </Button>
                </div>
              ) : (
                <Paragraph type="secondary">AI 认为当前写手方法论无需修改。</Paragraph>
              )
            ) : (
              <Alert
                type="info"
                showIcon
                message={
                  analysis.blogger_name
                    ? `「${analysis.blogger_name}」是内置写手，不支持直接修改写手配置，请参考上述建议。`
                    : '勾选的是【当前用户】手写报告，不支持应用到写手配置，请参考上述建议。'
                }
              />
            )}
          </div>
        ) : (
          <Empty description="正在加载…" />
        )}
      </Modal>
    </div>
  )
}

export default QualityLibPage
