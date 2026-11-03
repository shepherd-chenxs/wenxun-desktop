import { useState, useEffect } from 'react'
import { Layout, Card, Button, Table, Tag, Space, Divider, Modal, Form, Input, message, Popconfirm } from 'antd'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeftOutlined, FolderOpenOutlined, PlusOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons'
import type { TopicPack, Topic } from '../services/storeService'
import { storeService } from '../services/storeService'

const { Content } = Layout

const TopicPackDetailPage = () => {
  const { packId } = useParams<{ packId: string }>()
  const navigate = useNavigate()
  const [pack, setPack] = useState<TopicPack | null>(null)
  const [topics, setTopics] = useState<Topic[]>([])
  const [modalVisible, setModalVisible] = useState(false)
  const [editingTopic, setEditingTopic] = useState<Topic | null>(null)
  const [form] = Form.useForm()

  useEffect(() => {
    loadPackDetails()
  }, [packId])

  const loadPackDetails = async () => {
    try {
      const packs = await storeService.getTopicPacks()
      const foundPack = packs.find((p: TopicPack) => p.id === packId)
      setPack(foundPack || null)

      if (packId) {
        const loadedTopics = await storeService.getTopics(packId)
        setTopics(loadedTopics)
      }
    } catch (error) {
      console.error('加载话题包详情失败:', error)
      message.error('加载失败')
    }
  }

  const savePack = async (updatedPack: TopicPack, updatedTopics: Topic[]) => {
    try {
      // 保存话题包
      await storeService.saveTopicPack(updatedPack)
      
      // 删除当前话题包中已被移除的话题
      const existingTopics = await storeService.getTopics(packId || undefined)
      const topicsToDelete = existingTopics.filter(
        (t: Topic) => !updatedTopics.find((ut: Topic) => ut.id === t.id)
      )
      for (const topic of topicsToDelete) {
        await storeService.invoke('delete_topic', { id: topic.id })
      }
      
      // 保存或更新当前话题包的话题
      for (const topic of updatedTopics) {
        const existingTopic = existingTopics.find((t: Topic) => t.id === topic.id)
        if (existingTopic) {
          await storeService.invoke('update_topic', { topic })
        } else {
          await storeService.invoke('add_topic', { topic })
        }
      }

      setPack(updatedPack)
      setTopics(updatedTopics)
      message.success('保存成功')
    } catch (error) {
      console.error('保存话题包失败:', error)
      message.error('保存失败')
    }
  }

  const handleAddTopic = () => {
    setEditingTopic(null)
    form.resetFields()
    setModalVisible(true)
  }

  const handleEditTopic = (topic: Topic) => {
    setEditingTopic(topic)
    form.setFieldsValue({ content: topic.content })
    setModalVisible(true)
  }

  const handleDeleteTopic = (topicId: string) => {
    if (!pack) return
    const updatedTopics = topics.filter(t => t.id !== topicId)
    savePack(pack, updatedTopics)
    message.success('删除成功')
  }

  const handleSubmitTopic = (values: any) => {
    if (!pack) return

    if (editingTopic) {
      const updatedTopics = topics.map(t =>
        t.id === editingTopic.id ? { ...t, content: values.content } : t
      )
      savePack(pack, updatedTopics)
    } else {
      const newTopic: Topic = {
        id: Date.now().toString(),
        pack_id: pack.id!,
        content: values.content,
        status: 'available',
        created_at: new Date().toISOString()
      }
      savePack(pack, [...topics, newTopic])
    }
    setModalVisible(false)
  }

  return (
    <Layout style={{ minHeight: '100vh', background: '#f0f2f5' }}>
      <Content className="page-container">
        <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/topic-manage')} style={{ marginBottom: 16 }}>
          返回话题包列表
        </Button>

        {pack ? (
          <Card
            title={
              <Space>
                <FolderOpenOutlined />
                <span>{pack.name}</span>
              </Space>
            }
            extra={
              <Button type="primary" icon={<PlusOutlined />} onClick={handleAddTopic}>
                添加话题
              </Button>
            }
          >
            <div style={{ marginBottom: 12 }}>
              <strong>题材分类：</strong>{pack.category_name}
            </div>
            <div style={{ marginBottom: 12 }}>
              <strong>话题数量：</strong>{topics.length}
            </div>
            <Divider style={{ margin: '12px 0' }} />
            <div style={{ maxHeight: 'calc(100vh - 200px)', overflowY: 'auto' }}>
              <Table
                dataSource={topics}
                rowKey="id"
                pagination={false}
                columns={[
                  {
                    title: '序号',
                    width: 80,
                    render: (_, __, index) => index + 1,
                  },
                  {
                    title: '话题内容',
                    dataIndex: 'content',
                    ellipsis: true,
                  },
                  {
                    title: '状态',
                    dataIndex: 'status',
                    width: 100,
                    render: (status: string) => {
                      const statusMap: Record<string, string> = {
                        'available': '可用',
                        'used': '已用'
                      };
                      return <Tag color={status === 'available' ? 'green' : 'default'}>{statusMap[status] || status}</Tag>;
                    },
                  },
                  {
                    title: '创建时间',
                    dataIndex: 'created_at',
                    width: 180,
                    render: (time: string) => new Date(time).toLocaleString(),
                  },
                  {
                    title: '操作',
                    width: 180,
                    render: (_, topic) => (
                      <Space>
                        <Button type="link" icon={<EditOutlined />} onClick={() => handleEditTopic(topic)} size="small">
                          编辑
                        </Button>
                        <Popconfirm
                          title="确定删除此话题？"
                          onConfirm={() => handleDeleteTopic(topic.id!)}
                        >
                          <Button type="link" danger icon={<DeleteOutlined />} size="small">
                            删除
                          </Button>
                        </Popconfirm>
                      </Space>
                    ),
                  },
                ]}
              />
            </div>
          </Card>
        ) : (
          <Card>
            <div style={{ textAlign: 'center', padding: '40px 0' }}>
              <p style={{ color: '#999' }}>话题包不存在</p>
            </div>
          </Card>
        )}

        <Modal
          title={editingTopic ? '编辑话题' : '添加话题'}
          open={modalVisible}
          onCancel={() => setModalVisible(false)}
          footer={null}
        >
          <Form form={form} layout="vertical" onFinish={handleSubmitTopic}>
            <Form.Item
              label="话题内容"
              name="content"
              rules={[{ required: true, message: '请输入话题内容' }]}
            >
              <Input.TextArea rows={4} placeholder="请输入话题内容" />
            </Form.Item>
            <Form.Item>
              <Space>
                <Button type="primary" htmlType="submit">
                  保存
                </Button>
                <Button onClick={() => setModalVisible(false)}>取消</Button>
              </Space>
            </Form.Item>
          </Form>
        </Modal>
      </Content>
    </Layout>
  )
}

export default TopicPackDetailPage
