import { Table, Typography } from 'antd'
import { marked } from 'marked'

const { Text } = Typography

const DIM_LABELS = ['标题吸引力', '开头钩子', '结构节奏', '细节具体性', '人味', '总分']

interface ReportRow {
  key: string
  label: string
  score: number
  max: number
  isTotal?: boolean
}

function parseCells(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((c) => c.trim().replace(/\*\*/g, ''))
}

function scoreColor(score: number, max: number): string {
  const ratio = score / max
  if (ratio >= 0.75) return '#389e0d'
  if (ratio >= 0.5) return '#d48806'
  return '#cf1322'
}

function parseReport(report: string): {
  rows: ReportRow[]
  issues: string[]
  rounds: number | null
} | null {
  const lines = report.split(/\r?\n/)
  let headerIdx = -1
  for (let i = 0; i < lines.length - 2; i++) {
    if (!/^\s*\|.*\|\s*$/.test(lines[i])) continue
    if (!/^\s*\|[\s:\-|]+\|\s*$/.test(lines[i + 1])) continue
    if (!/^\s*\|.*\|\s*$/.test(lines[i + 2])) continue
    headerIdx = i
    break
  }
  if (headerIdx < 0) return null

  const values = parseCells(lines[headerIdx + 2])
  const nums = values.map((v) => Number(v.replace(/[^\d.-]/g, '')))
  if (nums.length < 6 || nums.some((n) => !Number.isFinite(n))) return null

  const rows: ReportRow[] = DIM_LABELS.map((label, i) => ({
    key: label,
    label,
    score: nums[i],
    max: i === nums.length - 1 ? 100 : 20,
    isTotal: i === nums.length - 1,
  }))

  // 遗留问题：表格之后的 “- xxx” 列表
  const issues: string[] = []
  let inIssues = false
  for (const line of lines) {
    if (inIssues) {
      const m = line.match(/^\s*[-*]\s+(.+)$/)
      if (m) issues.push(m[1].trim())
      else if (line.trim() === '') continue
      else if (/^[_*]?经\s*\d+\s*轮/.test(line.trim())) continue
      else break
    } else if (line.includes('遗留问题')) {
      inIssues = true
    }
  }

  const roundsMatch = report.match(/经\s*(\d+)\s*轮/)
  const rounds = roundsMatch ? Number(roundsMatch[1]) : null

  return { rows, issues, rounds }
}

const QualityReportView: React.FC<{ report: string }> = ({ report }) => {
  const parsed = parseReport(report)

  if (!parsed) {
    return (
      <div
        className="hot-analysis"
        dangerouslySetInnerHTML={{ __html: marked(report) as string }}
      />
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      <Table<ReportRow>
        rowKey="key"
        size="small"
        pagination={false}
        dataSource={parsed.rows}
        rowClassName={(r) => (r.isTotal ? 'quality-total-row' : '')}
        columns={[
          { title: '评分维度', dataIndex: 'label' },
          {
            title: '得分',
            dataIndex: 'score',
            render: (v: number, row: ReportRow) => (
              <Text strong style={{ color: row.isTotal ? '#1677ff' : scoreColor(v, row.max) }}>
                {v}
              </Text>
            ),
          },
          {
            title: '满分',
            dataIndex: 'max',
            render: (v: number) => <Text type="secondary">{v}</Text>,
          },
        ]}
      />

      {parsed.issues.length > 0 && (
        <div>
          <Text strong>遗留问题：</Text>
          <ul style={{ margin: '8px 0 0', paddingLeft: 20 }}>
            {parsed.issues.map((issue, i) => (
              <li key={i} style={{ marginBottom: 4 }}>
                <Text style={{ fontSize: 13 }}>{issue}</Text>
              </li>
            ))}
          </ul>
        </div>
      )}

      {parsed.rounds !== null && parsed.rounds > 0 && (
        <Text type="secondary" style={{ fontSize: 12 }}>
          经 {parsed.rounds} 轮体检-重写后选定最优版本
        </Text>
      )}
    </div>
  )
}

export default QualityReportView
