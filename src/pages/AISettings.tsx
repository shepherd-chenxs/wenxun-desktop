import { useState, useEffect } from 'react'
import { Layout, Input, Button, message, Card, Typography } from 'antd'
import { storeService } from '../services/storeService'

const { Content } = Layout
const { Title, Text, Link } = Typography

interface AISettingsProps {
  visible?: boolean
  onClose?: () => void
}

const AISettings: React.FC<AISettingsProps> = ({ visible = true, onClose }) => {
  const [apiKey, setApiKey] = useState('')
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    loadApiKey()
  }, [])

  const loadApiKey = async () => {
    try {
      const result = await storeService.getAIApiKey()
      setApiKey(result?.api_key || '')
    } catch (error) {
      console.error('加载API密钥失败:', error)
    }
  }

  const handleSave = async () => {
    if (!apiKey.trim()) {
      message.error('请输入API密钥')
      return
    }

    setLoading(true)
    try {
      const result = await storeService.saveAIApiKey({
        id: 'api_key',
        api_key: apiKey,
        created_at: new Date().toISOString()
      })
      if (result) {
        message.success('API密钥保存成功')
      }
    } catch (error: unknown) {
      message.error(`保存失败: ${error instanceof Error ? error.message : String(error)}`)
    } finally {
      setLoading(false)
    }
  }

  if (visible) {
    return (
      <Layout style={{ minHeight: '100vh', background: '#f0f2f5' }}>
        <Content className="page-container">
          <div style={{ maxWidth: 600, margin: '0 auto' }}>
            <Card title="AI 设置">
              <div style={{ marginBottom: 16 }}>
                <Title level={5}>DeepSeek API 配置</Title>
                <Text type="secondary">
                  请输入您的DeepSeek API密钥，获取地址：
                  <Link href="https://platform.deepseek.com/api_keys" target="_blank">
                    https://platform.deepseek.com/api_keys
                  </Link>
                </Text>
              </div>
              <div style={{ marginBottom: 24 }}>
                <Input.Password
                  placeholder="请输入DeepSeek API密钥"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  size="large"
                />
              </div>
              <div style={{ display: 'flex', gap: 12, justifyContent: 'flex-end' }}>
                {onClose && (
                  <Button onClick={onClose}>
                    取消
                  </Button>
                )}
                <Button type="primary" onClick={handleSave} loading={loading}>
                  保存
                </Button>
              </div>
            </Card>
          </div>
        </Content>
      </Layout>
    )
  }

  return null
}

export default AISettings
