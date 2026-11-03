import { useState, useEffect } from 'react'
import { Typography, Card, Button, Input, message } from 'antd'
import { FolderOpenOutlined } from '@ant-design/icons'
import { storeService } from '../services/storeService'
import wenxunIconUrl from '../assets/wenxun-icon.png'

const { Title, Text, Paragraph } = Typography

function errMsg(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  try {
    return JSON.stringify(e)
  } catch {
    return String(e)
  }
}

const AboutPage = () => {
  const [logPath, setLogPath] = useState('')

  useEffect(() => {
    storeService.getLogPath().then(setLogPath).catch(() => setLogPath('获取失败'))
  }, [])

  const handleOpenLogDir = async () => {
    try {
      await storeService.openLogDir()
    } catch (error) {
      message.error(errMsg(error))
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>关于</Title>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        <div style={{ maxWidth: 800, margin: '0 auto', display: 'flex', flexDirection: 'column', gap: 20 }}>

          <Card>
            <div style={{ textAlign: 'center', marginBottom: 16 }}>
              <img
                src={wenxunIconUrl}
                alt="文迅"
                style={{ width: 72, height: 72, objectFit: 'contain' }}
              />
              <Title level={3} style={{ margin: '12px 0 4px' }}>文迅</Title>
              <Text type="secondary">桌面文章编辑与发布工具</Text>
              <div style={{ marginTop: 8 }}>
                <Text type="secondary" style={{ fontSize: 13 }}>版本 2.5.1</Text>
              </div>
              <div style={{ marginTop: 4 }}>
                <Text type="secondary" style={{ fontSize: 12 }}>开源软件 · MIT License</Text>
              </div>
            </div>
          </Card>

          <Card title={<span><FolderOpenOutlined style={{ marginRight: 8 }} />日志文件</span>}>
            <Paragraph style={{ fontSize: 13, color: '#666', marginBottom: 8 }}>
              应用运行日志保存在本地文件中，遇到问题时可将日志文件提供给开发者排查。
            </Paragraph>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Input
                readOnly
                value={logPath || '加载中...'}
                style={{ flex: '1 1 620px', minWidth: 0, fontFamily: 'monospace', fontSize: 12 }}
              />
              <Button
                type="text"
                icon={<FolderOpenOutlined />}
                onClick={() => void handleOpenLogDir()}
              >
                打开目录
              </Button>
            </div>
          </Card>

          <Card title="免责声明">
            <Paragraph style={{ fontSize: 13, color: '#666', marginBottom: 12 }}>
              1. 本工具为辅助创作工具，AI 生成的文章内容仅供参考，不保证其准确性、完整性或合规性。发布前请务必进行人工审核。
            </Paragraph>
            <Paragraph style={{ fontSize: 13, color: '#666', marginBottom: 12 }}>
              2. 用户通过本工具发布到微信公众号的内容，需自行遵守《微信公众平台运营规范》。因内容违规导致的封号、处罚等后果，由用户自行承担。
            </Paragraph>
            <Paragraph style={{ fontSize: 13, color: '#666', marginBottom: 12 }}>
              3. 定时任务自动发布功能涉及自动化操作，用户应谨慎配置并定期检查任务执行情况，确保发布内容符合预期。
            </Paragraph>
            <Paragraph style={{ fontSize: 13, color: '#666', marginBottom: 0 }}>
              4. 本工具不会收集、存储或上传您的 API Key、公众号凭证等敏感信息至第三方服务器，所有数据均存储在本地。
            </Paragraph>
          </Card>

        </div>
      </div>
    </div>
  )
}

export default AboutPage
