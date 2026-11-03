import { useState, useEffect, useRef } from 'react'
import { Modal, Form, Input, Select, Button, message, Space } from 'antd'
import { UploadOutlined } from '@ant-design/icons'
import type { WechatAccount } from '../../services/storeService'
import { storeService } from '../../services/storeService'
import { markdownToStyledHtml } from '../../utils/typesetting'

const { TextArea } = Input

interface WechatPublishProps {
  visible: boolean
  title: string
  content: string
  templateId?: string
  onClose: () => void
  onSuccess: () => void
}

const WechatPublish = ({ visible, title, content, templateId = 'default', onClose, onSuccess }: WechatPublishProps) => {
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [accounts, setAccounts] = useState<WechatAccount[]>([])
  const fileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (visible) {
      loadAccounts()
    }
  }, [visible])

  const loadAccounts = async () => {
    try {
      const data = await storeService.getWechatAccounts()
      setAccounts(data || [])
      if (data && data.length > 0) {
        form.setFieldsValue({
          accountId: data[0].id,
          coverMediaId: data[0].default_cover_media_id || '',
        })
      }
    } catch (error) {
      console.error('加载公众号列表失败:', error)
    }
  }

  const handleSubmit = async (values: {
    accountId: string;
    title: string;
    author: string;
    digest: string;
    publishType: 'draft' | 'mass';
    coverMediaId?: string;
  }) => {
    const mediaId = values.coverMediaId?.trim() || ''
    if (!mediaId) {
      message.warning('请先上传封面素材')
      return
    }
    setLoading(true)
    try {
      // 将 Markdown 转换为带内联样式的 HTML (用于微信公众号)
      const styledHtml = markdownToStyledHtml(content, templateId)

      await storeService.wechatPublishArticle(
        values.accountId,
        {
          title: values.title,
          content: styledHtml,
          author: values.author || undefined,
          coverMediaId: mediaId,
          publishType: values.publishType,
        } as any
      )

      message.success('发布成功')
      onSuccess()
      onClose()
    } catch (error: any) {
      message.error(`发布失败: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleButtonClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const file = e.target.files[0]
      
      // 自动上传
      const accountId = form.getFieldValue('accountId')
      if (!accountId) {
        message.error('请先选择公众号')
        return
      }
      
      setUploading(true)
      try {
        // 将 File 转换为 ArrayBuffer
        const arrayBuffer = await file.arrayBuffer()
        const result = await storeService.wechatUploadMedia(accountId, {
          name: file.name,
          type: file.type,
          data: Array.from(new Uint8Array(arrayBuffer))
        })
        
        if (result.mediaId) {
          form.setFieldsValue({ coverMediaId: result.mediaId })
          message.success('上传成功')
        } else {
          message.error(`上传失败: ${result.error}`)
        }
      } catch (error: any) {
        message.error(`上传失败: ${error.message}`)
      } finally {
        setUploading(false)
      }
      
      // 清空 input，允许重复选择同一文件
      e.target.value = ''
    }
  }

  const handleAccountChange = (accountId: string) => {
    const selected = accounts.find((account) => account.id === accountId)
    form.setFieldsValue({
      accountId,
      coverMediaId: selected?.default_cover_media_id || '',
    })
  }

  return (
    <Modal
      title="发布到微信公众号"
      open={visible}
      onCancel={onClose}
      footer={null}
      width={600}
    >
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        onChange={handleFileChange}
        style={{ display: 'none' }}
        disabled={uploading}
      />
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          title: title,
          publishType: 'draft',
        }}
      >
        <Form.Item
          label="选择公众号"
          name="accountId"
          rules={[{ required: true, message: '请选择公众号' }]}
        >
          <Select placeholder="请选择要发布的公众号" onChange={handleAccountChange}>
            {accounts.map((account) => (
              <Select.Option key={account.id} value={account.id}>
                {account.name}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item
          label="文章标题"
          name="title"
          rules={[{ required: true, message: '请输入文章标题' }]}
        >
          <Input placeholder="请输入文章标题" />
        </Form.Item>

        <Form.Item
          label="作者"
          name="author"
        >
          <Input placeholder="请输入作者（可选）" />
        </Form.Item>

        <Form.Item
          label="摘要"
          name="digest"
        >
          <TextArea
            placeholder="请输入文章摘要（可选，不填则自动从正文中获取）"
            rows={3}
            maxLength={120}
            showCount
          />
        </Form.Item>

        <Form.Item
          label="封面素材"
          name="coverMediaId"
          required
          rules={[{ required: true, message: '请上传封面素材' }]}
          extra={<span style={{ fontSize: 12 }}>上传图片将用于文章封面</span>}
        >
          <Input
            readOnly
            placeholder="上传图片后自动填入素材ID"
            addonAfter={
              <Button
                type="text"
                icon={<UploadOutlined />}
                onClick={handleButtonClick}
                loading={uploading}
                disabled={uploading}
                style={{ padding: 0 }}
              >
                {uploading ? '上传中...' : '上传'}
              </Button>
            }
          />
        </Form.Item>

        <Form.Item
          label="发布类型"
          name="publishType"
          rules={[{ required: true, message: '请选择发布类型' }]}
        >
          <Select>
            <Select.Option value="draft">保存到草稿箱</Select.Option>
            <Select.Option value="mass">群发消息</Select.Option>
          </Select>
        </Form.Item>

        <Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={loading}>
              发布
            </Button>
            <Button onClick={onClose}>取消</Button>
          </Space>
        </Form.Item>
      </Form>
    </Modal>
  )
}

export default WechatPublish
