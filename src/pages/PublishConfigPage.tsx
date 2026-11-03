import { useState, useEffect, useRef } from 'react'
import { Layout, Tabs, Card, Button, Modal, Form, Input, message, Space, Popconfirm, Typography, Alert, Spin, List } from 'antd'
import {
  PlusOutlined, EditOutlined, DeleteOutlined, ReloadOutlined, GlobalOutlined, WechatOutlined,
  KeyOutlined, SafetyOutlined, CheckCircleOutlined, UploadOutlined,
} from '@ant-design/icons'
import type { WechatAccount as WechatAccountType } from '../services/storeService'
import { storeService } from '../services/storeService'

const { Content } = Layout
const { Title, Text, Link } = Typography

// ==================== 微信公众号配置面板 ====================
const WechatTab = () => {
  const [accounts, setAccounts] = useState<WechatAccountType[]>([])
  const [modalVisible, setModalVisible] = useState(false)
  const [editingAccount, setEditingAccount] = useState<WechatAccountType | null>(null)
  const [verifying, setVerifying] = useState(false)
  const [uploadingCover, setUploadingCover] = useState(false)
  const [defaultCoverMediaId, setDefaultCoverMediaId] = useState('')
  const [form] = Form.useForm()
  const coverFileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => { loadAccounts() }, [])

  const loadAccounts = async () => {
    try { setAccounts((await storeService.getWechatAccounts()) || []) } catch { /* ignore */ }
  }

  const handleAdd = () => {
    setEditingAccount(null)
    form.resetFields()
    setDefaultCoverMediaId('')
    setModalVisible(true)
  }
  const handleEdit = (account: WechatAccountType) => {
    setEditingAccount(account)
    form.setFieldsValue({
      name: account.name,
      appId: account.app_id,
      appSecret: account.app_secret,
    })
    setDefaultCoverMediaId(account.default_cover_media_id || '')
    setModalVisible(true)
  }
  const handleDelete = async (id: string) => {
    try {
      await storeService.deleteWechatAccount(id)
      message.success('已删除')
      loadAccounts()
    } catch (error: unknown) {
      message.error(`删除失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }
  const handleVerify = async () => {
    const appId = form.getFieldValue('appId')
    const appSecret = form.getFieldValue('appSecret')
    if (!appId || !appSecret) {
      message.warning('请先填写 AppID 和 AppSecret')
      return
    }
    setVerifying(true)
    try {
      const result = await storeService.verifyWechatAccount(appId, appSecret)
      message.success(result)
    } catch (e: any) {
      message.error(e?.message || String(e))
    } finally {
      setVerifying(false)
    }
  }

  const handleChooseCover = () => {
    const appId = form.getFieldValue('appId')
    const appSecret = form.getFieldValue('appSecret')
    if (!appId || !appSecret) {
      message.warning('请先填写 AppID 和 AppSecret')
      return
    }
    coverFileInputRef.current?.click()
  }

  const handleCoverFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    e.target.value = ''
    if (!file) return

    const appId = form.getFieldValue('appId')
    const appSecret = form.getFieldValue('appSecret')
    if (!appId || !appSecret) {
      message.error('请先填写 AppID 和 AppSecret')
      return
    }

    setUploadingCover(true)
    try {
      // 将 File 转换为 ArrayBuffer 后交给后端上传到微信永久素材库
      const arrayBuffer = await file.arrayBuffer()
      const result = await storeService.wechatUploadMediaWithCredentials(appId, appSecret, {
        name: file.name,
        type: file.type,
        data: Array.from(new Uint8Array(arrayBuffer)),
      })
      if (result.mediaId) {
        setDefaultCoverMediaId(result.mediaId)
        message.success('图片上传成功，素材 ID 已自动填入')
      } else {
        message.error(`上传失败: ${result.error}`)
      }
    } catch (error: unknown) {
      message.error(`上传失败: ${error instanceof Error ? error.message : String(error)}`)
    } finally {
      setUploadingCover(false)
    }
  }

  const handleSubmit = async (values: any) => {
    try {
      if (editingAccount) {
        await storeService.saveWechatAccount({
          ...editingAccount,
          name: values.name,
          app_id: values.appId,
          app_secret: values.appSecret,
          default_cover_media_id: defaultCoverMediaId.trim(),
        })
      } else {
        await storeService.saveWechatAccount({
          id: Date.now().toString(),
          name: values.name,
          app_id: values.appId,
          app_secret: values.appSecret,
          default_cover_media_id: defaultCoverMediaId.trim(),
          status: 'active',
          created_at: new Date().toISOString(),
        })
      }
      message.success(editingAccount ? '已更新' : '已添加')
      setModalVisible(false)
      loadAccounts()
    } catch (error: unknown) {
      message.error(`保存失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  return (
    <>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <Title level={5} style={{ margin: 0 }}>已配置的公众号</Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>添加公众号</Button>
      </div>

      {accounts.length === 0 ? (
        <Card><div style={{ textAlign: 'center', padding: 40, color: '#999' }}>暂无公众号，点击右上角添加</div></Card>
      ) : (
        <List
          dataSource={accounts}
          style={{ background: '#fff', borderRadius: 8 }}
          renderItem={account => (
            <List.Item
              style={{ padding: '16px 24px' }}
              actions={[
                <Button key="edit" type="link" icon={<EditOutlined />} onClick={() => handleEdit(account)}>编辑</Button>,
                <Popconfirm key="del" title="确定删除？" onConfirm={() => handleDelete(account.id)}>
                  <Button type="link" danger icon={<DeleteOutlined />}>删除</Button>
                </Popconfirm>,
              ]}
            >
              <List.Item.Meta
                avatar={<WechatOutlined style={{ fontSize: 20, color: '#07c160' }} />}
                title={<span style={{ fontWeight: 500 }}>{account.name}</span>}
                description={
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    AppID: {account.app_id}
                  </Text>
                }
              />
            </List.Item>
          )}
        />
      )}

      <Modal title={editingAccount ? '编辑公众号' : '添加公众号'} open={modalVisible} onCancel={() => setModalVisible(false)} footer={null}>
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item label="公众号名称" name="name" rules={[{ required: true, message: '请输入公众号名称' }]}>
            <Input placeholder="例如：我的公众号" />
          </Form.Item>
          <Form.Item label="AppID" name="appId" rules={[{ required: true, message: '请输入AppID' }]}>
            <Input placeholder="请输入微信公众号 AppID" />
          </Form.Item>
          <Form.Item label="AppSecret" name="appSecret" rules={[{ required: true, message: '请输入AppSecret' }]}>
            <Input.Password placeholder="请输入微信公众号 AppSecret" />
          </Form.Item>
          <Form.Item
            label="默认封面素材ID"
            extra={
              <span style={{ fontSize: 12 }}>
                该图片将作为未单独指定封面时的默认封面。
              </span>
            }
          >
            <Space.Compact style={{ width: '100%' }}>
              <Input
                value={defaultCoverMediaId}
                onChange={(e) => setDefaultCoverMediaId(e.target.value)}
                placeholder="上传图片后自动填入素材ID"
                readOnly
              />
              <input
                ref={coverFileInputRef}
                type="file"
                accept="image/*"
                onChange={handleCoverFileChange}
                style={{ display: 'none' }}
                disabled={uploadingCover}
              />
              <Button
                icon={<UploadOutlined />}
                onClick={handleChooseCover}
                loading={uploadingCover}
                disabled={uploadingCover}
              >
                {uploadingCover ? '上传中...' : '上传'}
              </Button>
            </Space.Compact>
          </Form.Item>
          <Form.Item style={{ marginBottom: 16 }}>
            <Button icon={<CheckCircleOutlined />} onClick={handleVerify} loading={verifying} block>
              验证密钥
            </Button>
          </Form.Item>
          <Form.Item style={{ marginBottom: 16 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              AppID 和 AppSecret 可在微信开发者平台获取：
              <Link href="https://developers.weixin.qq.com/platform" target="_blank">developers.weixin.qq.com/platform</Link>
              （登录后进入「公众号 → 基础信息 → 开发密钥」）
            </Text>
          </Form.Item>
          <Form.Item style={{ marginBottom: 0 }}>
            <Space><Button type="primary" htmlType="submit">保存</Button><Button onClick={() => setModalVisible(false)}>取消</Button></Space>
          </Form.Item>
        </Form>
      </Modal>
    </>
  )
}

// ==================== AI 配置面板 ====================
const AITab = () => {
  const [apiKey, setApiKey] = useState('')
  const [loading, setLoading] = useState(false)

  useEffect(() => { loadApiKey() }, [])

  const loadApiKey = async () => {
    try { setApiKey((await storeService.getAIApiKey())?.api_key || '') } catch { /* ignore */ }
  }
  const handleSave = async () => {
    if (!apiKey.trim()) { message.error('请输入 API Key'); return }
    setLoading(true)
    try {
      await storeService.saveAIApiKey({ id: 'api_key', api_key: apiKey, created_at: new Date().toISOString() })
      message.success('已保存')
    } catch (e: unknown) {
      message.error(`保存失败: ${e instanceof Error ? e.message : String(e)}`)
    } finally { setLoading(false) }
  }

  return (
    <>
      <Title level={5} style={{ marginBottom: 16 }}>API Key 配置</Title>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20 }}>
        {/* 说明卡片 */}
        <Card>
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <KeyOutlined style={{ fontSize: 24, color: '#1677ff' }} />
              <div>
                <Text strong style={{ fontSize: 16 }}>DeepSeek API Key</Text><br />
                <Text type="secondary">用于 AI 生成文章和标题</Text>
              </div>
            </div>
            <div style={{ lineHeight: 2 }}>
              <Text strong>获取方式</Text><br />
              <Link href="https://platform.deepseek.com/api_keys" target="_blank">
                platform.deepseek.com/api_keys
              </Link>
            </div>
            <Text type="secondary">
              注册 DeepSeek 账号后，在 API Keys 页面创建密钥，复制后粘贴到右侧输入框保存即可。
            </Text>
          </Space>
        </Card>

        {/* 配置卡片 */}
        <Card title="密钥配置">
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <div>
              <Text strong style={{ display: 'block', marginBottom: 8 }}>API Key</Text>
              <Input.Password
                placeholder="sk-xxxxxxxxxxxxxxxx"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                size="large"
              />
            </div>
            <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
              <Space>
                {apiKey && <Text type="secondary" style={{ fontSize: 12 }}>已填写</Text>}
                <Button type="primary" onClick={handleSave} loading={loading}>保存</Button>
              </Space>
            </div>
          </Space>
        </Card>
      </div>
    </>
  )
}

// ==================== IP 白名单配置面板 ====================
const IPWhitelistTab = () => {
  const [publicIP, setPublicIP] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const fetchIP = async () => {
    setLoading(true); setError('')
    try { setPublicIP(await storeService.getPublicIP()) } catch { setError('获取失败，请检查网络') }
    finally { setLoading(false) }
  }

  useEffect(() => { fetchIP() }, [])

  return (
    <>
      <Title level={5} style={{ marginBottom: 16 }}>IP 白名单配置</Title>

      <Alert
        type="info" showIcon
        style={{ marginBottom: 20 }}
        message="为什么需要配置？"
        description="微信公众号要求调用 API 的服务器 IP 必须在白名单中，否则无法获取 access_token，发布文章会失败。"
      />

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20, marginBottom: 20 }}>
        {/* 当前 IP */}
        <Card
          title={<Space><GlobalOutlined />当前公网 IP</Space>}
          extra={<Button icon={<ReloadOutlined />} onClick={fetchIP} loading={loading} size="small">刷新</Button>}
        >
          {error ? <Text type="danger">{error}</Text> :
           loading ? <Spin /> :
           <Text copyable style={{ fontSize: 28, fontWeight: 700, fontFamily: 'monospace' }}>{publicIP}</Text>}
        </Card>

        {/* 平台入口 */}
        <Card title="管理平台">
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <div>
              <Text strong>微信公众号后台</Text><br />
              <Link href="https://mp.weixin.qq.com" target="_blank">mp.weixin.qq.com</Link>
            </div>
            <div style={{ lineHeight: 2 }}>
              <Text strong>配置路径</Text><br />
              <Text type="secondary">左侧菜单：设置与开发 → 安全中心 → IP 白名单</Text>
            </div>
          </Space>
        </Card>
      </div>

      <Alert type="warning" showIcon message="服务器 IP 变化后需重新更新白名单，否则 API 调用将失败。" />
    </>
  )
}

// ==================== 发布配置页面 ====================
const PublishConfigPage = () => {
  const tabItems = [
    { key: 'wechat', label: <Space><WechatOutlined />微信公众号</Space>, children: <WechatTab /> },
    { key: 'ip-whitelist', label: <Space><SafetyOutlined />IP 白名单</Space>, children: <IPWhitelistTab /> },
    { key: 'ai', label: <Space><KeyOutlined />AI 设置</Space>, children: <AITab /> },
  ]

  return (
    <Layout style={{ height: '100%', background: '#f0f2f5' }}>
      <Content style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
        <div style={headerStyle}>
          <Title level={4} style={{ margin: 0 }}>发布配置</Title>
        </div>
        <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
          <Card style={{ minHeight: '100%' }}>
            <Tabs items={tabItems} />
          </Card>
        </div>
      </Content>
    </Layout>
  )
}

const headerStyle: React.CSSProperties = {
  flexShrink: 0,
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0',
}

export default PublishConfigPage
