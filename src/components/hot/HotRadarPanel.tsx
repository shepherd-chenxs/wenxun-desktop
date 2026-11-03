/**
 * 热点雷达：聚合各平台公开热榜（微博/知乎/抖音/百度/头条/B站），
 * 选题 → AI 成文 → 存入内容工作台，不依赖任何公众号后台凭证。
 */
import { useEffect, useState } from 'react'
import {
  Button, Empty, Input, List, Popover, Select, Space, Spin, Tag, Typography, message,
} from 'antd'
import { FireOutlined, LinkOutlined, QuestionCircleOutlined, ReloadOutlined, SaveOutlined } from '@ant-design/icons'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { getErrorMessage } from '../../utils/getErrorMessage'

const { Text } = Typography

interface HotItem {
  rank: number
  title: string
  heat: string
  url: string
}

interface HotSource {
  key: string
  name: string
}

interface TrackBlogger {
  id: string
  name: string
  preset_key: string
  is_builtin?: boolean
}

interface GeneratedArticle {
  title: string
  content: string
  report: string
  score: number
  rounds: number
}

const DEFAULT_WORD_COUNT = 1500
const HOT_RADAR_HELP = '各平台实时热榜，无需登录任何账号。选好话题后点「AI 成文」，会按所选写手自动生成并体检打分，达标才入库，结果存入内容工作台草稿。'

function formatHeat(heat: string): string {
  if (!heat) return ''
  const n = Number(heat)
  if (Number.isNaN(n)) return heat
  if (n >= 100000000) return `${(n / 100000000).toFixed(1)}亿`
  if (n >= 10000) return `${(n / 10000).toFixed(1)}万`
  return heat
}

export default function HotRadarPanel() {
  const [sources, setSources] = useState<HotSource[]>([])
  const [source, setSource] = useState('weibo')
  const [items, setItems] = useState<HotItem[]>([])
  const [loading, setLoading] = useState(false)
  const [keyword, setKeyword] = useState('')
  const [busyTitle, setBusyTitle] = useState<string | null>(null)
  const [tracks, setTracks] = useState<TrackBlogger[]>([])
  const [track, setTrack] = useState('emotion')

  useEffect(() => {
    tauriInvoke<HotSource[]>('get_hot_sources')
      .then((list) => {
        setSources(list)
        if (list.length > 0) setSource(list[0].key)
      })
      .catch((e) => message.error(getErrorMessage(e)))
    tauriInvoke<TrackBlogger[]>('get_bloggers')
      .then((list) => {
        setTracks(list)
        const builtin = list.filter((b) => b.is_builtin)
        const initial = builtin.length > 0 ? builtin[0] : list[0]
        if (initial) setTrack(initial.id)
      })
      .catch(() => {})
  }, [])

  const load = async (src: string) => {
    setLoading(true)
    setItems([])
    try {
      setItems(await tauriInvoke<HotItem[]>('get_hot_list', { source: src }))
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (source) void load(source)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [source])

  const filtered = keyword.trim()
    ? items.filter((it) => it.title.toLowerCase().includes(keyword.trim().toLowerCase()))
    : items

  const openTopicUrl = async (item: HotItem) => {
    if (!item.url) {
      message.info('该来源暂无原文链接')
      return
    }
    try {
      await tauriInvoke('open_url', { url: item.url })
    } catch (e) {
      message.error(getErrorMessage(e))
    }
  }

  const handleGenerate = async (item: HotItem) => {
    setBusyTitle(item.title)
    try {
      const r = await tauriInvoke<GeneratedArticle>(
        'generate_blogger_article',
        { bloggerId: track, platform: sources.find((s) => s.key === source)?.name || '', topic: item.title, wordCount: DEFAULT_WORD_COUNT },
      )
      await tauriInvoke('save_article', {
        article: {
          id: Date.now().toString(),
          title: r.title,
          content: r.content,
          summary: r.content.replace(/[#*\n]/g, '').slice(0, 60),
          status: 'draft',
          folder_id: null,
          wechat_article_id: null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          quality_report: `质量体检 ${r.score} 分${r.rounds > 0 ? `（${r.rounds} 轮自动重写）` : '（一次达标）'}\n\n${r.report}`,
        },
      })
      message.success({ content: `《${r.title}》已生成（体检 ${r.score} 分）并存入内容工作台草稿`, duration: 4 })
    } catch (e) {
      message.error(getErrorMessage(e))
    } finally {
      setBusyTitle(null)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <style>{'.hot-radar-pagination .ant-pagination-item { margin-inline-end: 16px !important; }'}</style>
      <div
        style={{
          display: 'flex', alignItems: 'center', gap: 8,
          padding: '6px 10px', borderRadius: 6,
          border: '1px solid #95de64', background: '#fcfff5',
        }}
      >
        <Text style={{ fontSize: 13, whiteSpace: 'nowrap' }}>选择虚拟写手</Text>
        <Select
          value={track}
          onChange={(v) => setTrack(v)}
          options={tracks.map((t) => ({ value: t.id, label: t.name }))}
          showSearch
          optionFilterProp="label"
          placeholder="搜索或选择写手"
          style={{ width: 340 }}
        />
        <Popover
          trigger="click"
          content={
            <Text style={{ fontSize: 13 }}>
              点击「AI 成文」将使用该虚拟写手进行撰文
            </Text>
          }
        >
          <QuestionCircleOutlined
            style={{ fontSize: 14, color: '#8c8c8c', cursor: 'pointer' }}
          />
        </Popover>
      </div>
      <Space wrap>
        <Select
          value={source}
          onChange={(v) => setSource(v)}
          options={sources.map((s) => ({ value: s.key, label: s.name }))}
          style={{ width: 110 }}
        />
        <Input
          allowClear
          placeholder="按关键词过滤话题"
          prefix={<Text type="secondary">过滤</Text>}
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          style={{ width: 240 }}
        />
        <Button icon={<ReloadOutlined />} loading={loading} onClick={() => void load(source)}>
          刷新
        </Button>
        <Popover
          trigger="click"
          content={
            <div style={{ maxWidth: 420 }}>
              <Text style={{ fontSize: 13 }}>{HOT_RADAR_HELP}</Text>
            </div>
          }
        >
          <Button icon={<QuestionCircleOutlined />}>帮助</Button>
        </Popover>
      </Space>
      {loading ? (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin tip="正在获取热榜…" />
        </div>
      ) : filtered.length === 0 ? (
        <Empty description={keyword ? '没有匹配的话题' : '暂无数据'} />
      ) : (
        <List
          size="small"
          dataSource={filtered}
          pagination={filtered.length > 20
            ? { pageSize: 20, size: 'small', showSizeChanger: false, className: 'hot-radar-pagination' }
            : false}
          renderItem={(item) => (
            <List.Item
              style={{ padding: '6px 4px' }}
              actions={[
                item.url ? (
                  <Button
                    key="open"
                    size="small"
                    type="link"
                    icon={<LinkOutlined />}
                    onClick={() => void openTopicUrl(item)}
                  >
                    原文
                  </Button>
                ) : null,
                <Button
                  key="gen"
                  size="small"
                  type="primary"
                  ghost
                  icon={<FireOutlined />}
                  loading={busyTitle === item.title}
                  onClick={() => void handleGenerate(item)}
                >
                  AI 成文
                </Button>,
              ]}
            >
              <Space size={8} align="baseline" style={{ minWidth: 0, flex: 1 }}>
                <Text
                  type={item.rank <= 3 ? 'danger' : 'secondary'}
                  style={{ fontSize: 12, width: 20, textAlign: 'left', flexShrink: 0, whiteSpace: 'nowrap', display: 'inline-block' }}
                >
                  {item.rank}
                </Text>
                <Text style={{ fontSize: 13, wordBreak: 'break-all' }}>{item.title}</Text>
                {item.heat && (
                  <Tag color="volcano" style={{ fontSize: 11, flexShrink: 0, marginRight: 0 }}>
                    🔥 {formatHeat(item.heat)}
                  </Tag>
                )}
              </Space>
            </List.Item>
          )}
        />
      )}
      {items.length > 0 && keyword.trim() && (
        <Text type="secondary" style={{ fontSize: 12 }}>
          共 {items.length} 条，匹配 {filtered.length} 条
        </Text>
      )}
      <Text type="secondary" style={{ fontSize: 12 }}>
        <SaveOutlined /> 生成结果以草稿形式写入「内容工作台-文章列表」，可再编辑后发布；
        <LinkOutlined /> 点击话题右侧「原文」可在对应平台查看完整语境。
      </Text>
    </div>
  )
}
