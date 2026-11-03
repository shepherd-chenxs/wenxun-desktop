import { useState, useEffect } from 'react'
import { List, Button, message, Empty } from 'antd'
import { DeleteOutlined, ReloadOutlined } from '@ant-design/icons'
import { storeService, type GenerationHistory } from '../../services/storeService'

interface AIGenerationHistoryProps {
  onSelect: (content: string) => void
  onClose: () => void
}

const AIGenerationHistory: React.FC<AIGenerationHistoryProps> = ({ onSelect, onClose }) => {
  const [history, setHistory] = useState<GenerationHistory[]>([])

  useEffect(() => {
    loadHistory()
  }, [])

  const loadHistory = async () => {
    try {
      const historyData = await storeService.getGenerationHistory()
      setHistory(historyData)
    } catch (error) {
      console.error('加载生成历史失败:', error)
    }
  }

  const handleClearHistory = async () => {
    try {
      await storeService.clearGenerationHistory()
      setHistory([])
      message.success('历史记录已清空')
    } catch (error) {
      console.error('清空历史记录失败:', error)
      message.error('清空历史记录失败')
    }
  }

  const handleSelectHistory = (item: GenerationHistory) => {
    onSelect(item.content)
    onClose()
  }

  return (
    <div style={{ padding: 20, maxHeight: 500, overflowY: 'auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 20 }}>
        <h3>生成历史</h3>
        <Button
          danger
          icon={<DeleteOutlined />}
          onClick={handleClearHistory}
          disabled={history.length === 0}
        >
          清空历史
        </Button>
      </div>

      {history.length === 0 ? (
        <Empty description="暂无生成历史" />
      ) : (
        <List
          itemLayout="vertical"
          dataSource={history}
          renderItem={(item) => (
            <List.Item
              actions={[
                <Button
                  key="select"
                  type="primary"
                  icon={<ReloadOutlined />}
                  onClick={() => handleSelectHistory(item)}
                >
                  再次使用
                </Button>
              ]}
            >
              <List.Item.Meta
                title={
                  <div>
                    <span style={{ fontWeight: 'bold' }}>主题：{item.topic}</span>
                  </div>
                }
                description={
                  <div>
                    <div style={{ marginBottom: 8, color: '#999', fontSize: 12 }}>
                      生成时间：{new Date(item.created_at).toLocaleString()}
                    </div>
                    <div style={{ lineHeight: 1.5, maxHeight: 80, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {item.content.replace(/<[^>]+>/g, '').substring(0, 100)}...
                    </div>
                  </div>
                }
              />
            </List.Item>
          )}
        />
      )}
    </div>
  )
}

export default AIGenerationHistory
