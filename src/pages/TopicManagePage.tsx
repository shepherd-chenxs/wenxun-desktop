import { useState, useEffect, useCallback } from 'react'
import { Layout, Button, Modal, Form, Input, Select, message, Space, Popconfirm, Tag, List, Badge, Empty, Typography } from 'antd'
import { useNavigate } from 'react-router-dom'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  FolderOpenOutlined,
  EyeOutlined,
  AppstoreOutlined,
} from '@ant-design/icons'
import type { TopicCategory, TopicPack } from '../services/storeService'
import { storeService } from '../services/storeService'

const { Content } = Layout
const { TextArea } = Input
const { Title, Text } = Typography

const DEFAULT_CATEGORY_PREFIX = 'category_'
const SIDER_WIDTH = 280

const TopicManagePage = () => {
  const navigate = useNavigate()

  const [categories, setCategories] = useState<TopicCategory[]>([])
  const [packs, setPacks] = useState<TopicPack[]>([])
  const [topicCountMap, setTopicCountMap] = useState<Record<string, number>>({})
  const [selectedCategoryId, setSelectedCategoryId] = useState<string>('__all__')

  const [categoryModalVisible, setCategoryModalVisible] = useState(false)
  const [editingCategory, setEditingCategory] = useState<TopicCategory | null>(null)
  const [categoryForm] = Form.useForm()

  const [packModalVisible, setPackModalVisible] = useState(false)
  const [editingPack, setEditingPack] = useState<TopicPack | null>(null)
  const [packForm] = Form.useForm()

  const loadAll = useCallback(async () => {
    try {
      const [cats, allPacks, allTopics] = await Promise.all([
        storeService.getTopicCategories(),
        storeService.getTopicPacks(),
        storeService.getTopics(),
      ])
      setCategories(cats || [])
      setPacks((allPacks || []).map(p => ({ ...p, id: p.id || '' })))
      const countMap: Record<string, number> = {}
      for (const t of allTopics || []) {
        if (t.pack_id) countMap[t.pack_id] = (countMap[t.pack_id] || 0) + 1
      }
      setTopicCountMap(countMap)
    } catch (error) {
      console.error('加载数据失败:', error)
    }
  }, [])

  useEffect(() => { loadAll() }, [loadAll])

  const packCountByCategory = (catId: string) => packs.filter(p => p.category_id === catId).length
  const filteredPacks = selectedCategoryId === '__all__' ? packs : packs.filter(p => p.category_id === selectedCategoryId)
  const selectedCategory = categories.find(c => c.id === selectedCategoryId)
  const isDefaultCategory = (id: string) => id.startsWith(DEFAULT_CATEGORY_PREFIX)

  // --- Category CRUD ---
  const handleAddCategory = () => { setEditingCategory(null); categoryForm.resetFields(); setCategoryModalVisible(true) }
  const handleEditCategory = (cat: TopicCategory) => {
    setEditingCategory(cat)
    categoryForm.setFieldsValue({ name: cat.name, description: cat.description })
    setCategoryModalVisible(true)
  }
  const handleDeleteCategory = async (id: string) => {
    try {
      for (const pack of packs.filter(p => p.category_id === id)) {
        await storeService.invoke('delete_topic_pack', { id: pack.id })
      }
      await storeService.invoke('delete_topic_category', { id })
      if (selectedCategoryId === id) setSelectedCategoryId('__all__')
      message.success('分类已删除')
      await loadAll()
    } catch (error: unknown) {
      message.error(`删除失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }
  const handleCategorySubmit = async (values: any) => {
    try {
      if (editingCategory) {
        await storeService.invoke('update_topic_category', { category: { id: editingCategory.id, name: values.name, description: values.description || '', created_at: editingCategory.created_at } })
        message.success('分类已更新')
      } else {
        await storeService.invoke('add_topic_category', { category: { id: Date.now().toString(), name: values.name, description: values.description || '', created_at: new Date().toISOString() } })
        message.success('分类已添加')
      }
      setCategoryModalVisible(false)
      await loadAll()
    } catch (error: unknown) {
      message.error(`保存失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  // --- Pack CRUD ---
  const handleAddPack = () => { setEditingPack(null); packForm.resetFields(); if (selectedCategoryId !== '__all__') packForm.setFieldsValue({ categoryId: selectedCategoryId }); setPackModalVisible(true) }
  const handleEditPack = (pack: TopicPack) => { setEditingPack(pack); packForm.setFieldsValue({ categoryId: pack.category_id, name: pack.name }); setPackModalVisible(true) }
  const handleDeletePack = async (id: string) => {
    try {
      await storeService.invoke('delete_topic_pack', { id })
      message.success('题材包已删除')
      await loadAll()
    } catch (error: unknown) {
      message.error(`删除失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }
  const handlePackSubmit = async (values: any) => {
    try {
      const category = categories.find(c => c.id === values.categoryId)
      const now = new Date().toISOString()
      if (editingPack) {
        await storeService.invoke('update_topic_pack', { pack: { ...editingPack, category_id: values.categoryId, category_name: category?.name || '', name: values.name } })
        message.success('题材包已更新')
      } else {
        const topicNames = (values.topics || '').split('\n').map((t: string) => t.trim()).filter((t: string) => t)
        const packId = Date.now().toString()
        await storeService.invoke('add_topic_pack', { pack: { id: packId, category_id: values.categoryId, category_name: category?.name || '', name: values.name, description: '', topics: topicNames.join('\n'), created_at: now } })
        for (let i = 0; i < topicNames.length; i++) {
          await storeService.invoke('add_topic', { topic: { id: `${packId}_${i}`, pack_id: packId, content: topicNames[i], status: 'available', created_at: now } })
        }
        message.success('题材包已创建')
      }
      setPackModalVisible(false)
      await loadAll()
    } catch (error: unknown) {
      message.error(`保存失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      {/* ====== 页面标题 ====== */}
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>题材管理</Title>
      </div>
      <div style={{ flexShrink: 0, height: 16, background: '#f0f2f5' }} />
      {/* ====== 主体布局 ====== */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden', padding: '0 16px 16px 16px', gap: 16 }}>
        {/* 左侧边栏 */}
        <div style={{ width: SIDER_WIDTH, flexShrink: 0, background: '#fff', borderRadius: 8, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <div style={siderHeaderStyle}>
            <Title level={5} style={{ margin: 0 }}>题材分类</Title>
            <Button type="primary" size="small" icon={<PlusOutlined />} onClick={handleAddCategory}>新增</Button>
          </div>
        <div style={{ flex: 1, overflowY: 'auto', padding: '8px 0' }}>
          {/* 全部 */}
          <div onClick={() => setSelectedCategoryId('__all__')} style={siderItemStyle(selectedCategoryId === '__all__')}>
            <Space><AppstoreOutlined /><span>全部题材</span></Space>
            <Badge count={packs.length} overflowCount={999} style={{ backgroundColor: '#999' }} />
          </div>
          {categories.map(cat => (
            <div key={cat.id} onClick={() => setSelectedCategoryId(cat.id)} style={siderItemStyle(selectedCategoryId === cat.id)}>
              <div style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: 14 }}>{cat.name}</div>
              <Space size={2} style={{ flexShrink: 0, marginLeft: 8 }}>
                <Badge count={packCountByCategory(cat.id)} overflowCount={99} style={{ backgroundColor: '#1677ff' }} />
                {!isDefaultCategory(cat.id) && (
                  <Button type="text" size="small" icon={<EditOutlined />} onClick={e => { e.stopPropagation(); handleEditCategory(cat) }} style={{ color: '#999' }} />
                )}
                {!isDefaultCategory(cat.id) && (
                  <Popconfirm title="删除分类将同时删除其下所有题材包，确定删除？" onConfirm={e => { e?.stopPropagation(); handleDeleteCategory(cat.id) }} onCancel={e => e?.stopPropagation()}>
                    <Button type="text" size="small" danger icon={<DeleteOutlined />} onClick={e => e.stopPropagation()} />
                  </Popconfirm>
                )}
              </Space>
            </div>
          ))}
        </div>
      </div>

      {/* ====== 右侧内容 ====== */}
      <Content style={{ display: 'flex', flexDirection: 'column', flex: 1, overflow: 'hidden', background: '#fff', borderRadius: 8 }}>
        {/* 固定标题栏 */}
        <div style={contentHeaderStyle}>
          <Title level={4} style={{ margin: 0 }}>
            {selectedCategoryId === '__all__' ? '全部题材包' : `${selectedCategory?.name || ''} — 题材包`}
            <Text type="secondary" style={{ fontSize: 14, marginLeft: 8, fontWeight: 400 }}>({filteredPacks.length})</Text>
          </Title>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleAddPack}>添加题材包</Button>
        </div>

        {/* 可滚动内容 */}
        <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
          {filteredPacks.length === 0 ? (
            <div style={{ background: '#fff', borderRadius: 8, padding: '60px 0', textAlign: 'center' }}>
              <Empty description="暂无题材包" />
            </div>
          ) : (
            <List
              dataSource={filteredPacks}
              style={{ background: '#fff', borderRadius: 8 }}
              renderItem={pack => (
                <List.Item
                  key={pack.id}
                  style={{ padding: '16px 24px' }}
                  actions={[
                    <Button key="view" type="link" icon={<EyeOutlined />} onClick={() => navigate(`/topic-manage/packs/${pack.id}`)}>查看</Button>,
                    <Button key="edit" type="link" icon={<EditOutlined />} onClick={() => handleEditPack(pack)}>编辑</Button>,
                    <Popconfirm key="del" title="确定删除此题材包？" onConfirm={() => handleDeletePack(pack.id || '')}>
                      <Button type="link" danger icon={<DeleteOutlined />}>删除</Button>
                    </Popconfirm>,
                  ]}
                >
                  <List.Item.Meta
                    avatar={<FolderOpenOutlined style={{ fontSize: 20, color: '#1677ff' }} />}
                    title={<Space><span style={{ fontWeight: 500 }}>{pack.name}</span><Tag color="blue">{pack.category_name}</Tag></Space>}
                    description={
                      <Space size={16}>
                        <Text type="secondary">话题: {topicCountMap[pack.id || ''] || 0}</Text>
                        <Text type="secondary">{pack.created_at ? new Date(pack.created_at).toLocaleDateString('zh-CN') : ''}</Text>
                      </Space>
                    }
                  />
                </List.Item>
              )}
            />
          )}
        </div>

        {/* ====== 分类弹窗 ====== */}
        <Modal title={editingCategory ? '编辑分类' : '添加分类'} open={categoryModalVisible} onCancel={() => setCategoryModalVisible(false)} footer={null}>
          <Form form={categoryForm} layout="vertical" onFinish={handleCategorySubmit}>
            <Form.Item label="分类名称" name="name" rules={[{ required: true, message: '请输入分类名称' }]}>
              <Input placeholder="例如：科技、生活、教育" />
            </Form.Item>
            <Form.Item label="分类描述" name="description" rules={[{ required: true, message: '请输入分类描述' }]}>
              <TextArea placeholder="请简要描述该分类的内容范围" rows={3} />
            </Form.Item>
            <Form.Item>
              <Space><Button type="primary" htmlType="submit">保存</Button><Button onClick={() => setCategoryModalVisible(false)}>取消</Button></Space>
            </Form.Item>
          </Form>
        </Modal>

        {/* ====== 题材包弹窗 ====== */}
        <Modal title={editingPack ? '编辑题材包' : '新建题材包'} open={packModalVisible} onCancel={() => setPackModalVisible(false)} footer={null} width={600}>
          <Form form={packForm} layout="vertical" onFinish={handlePackSubmit}>
            <Form.Item label="题材分类" name="categoryId" rules={[{ required: true, message: '请选择题材分类' }]}>
              <Select placeholder="请选择题材分类">
                {categories.map(cat => (<Select.Option key={cat.id} value={cat.id}>{cat.name}</Select.Option>))}
              </Select>
            </Form.Item>
            <Form.Item label="题材包名称" name="name" rules={[{ required: true, message: '请输入题材包名称' }]}>
              <Input placeholder="例如：春节热点话题包" />
            </Form.Item>
            {!editingPack && (
              <Form.Item label="话题列表" name="topics" rules={[{ required: true, message: '请输入话题列表' }]} extra="每行一个话题">
                <TextArea placeholder="例如：&#10;人工智能的10个发展趋势&#10;AI如何改变生活" rows={8} />
              </Form.Item>
            )}
            <Form.Item>
              <Space><Button type="primary" htmlType="submit">{editingPack ? '更新' : '创建'}</Button><Button onClick={() => setPackModalVisible(false)}>取消</Button></Space>
            </Form.Item>
          </Form>
        </Modal>
      </Content>
      </div>
    </div>
  )
}

// ====== 统一样式 ======
const siderHeaderStyle: React.CSSProperties = {
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '16px', borderBottom: '1px solid #f0f0f0',
}

const siderItemStyle = (active: boolean): React.CSSProperties => ({
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '10px 12px', margin: '2px 8px', borderRadius: 6,
  cursor: 'pointer',
  background: active ? '#e6f4ff' : 'transparent',
  color: active ? '#1677ff' : 'inherit',
  fontWeight: active ? 500 : 400,
  transition: 'all 0.2s',
})

const contentHeaderStyle: React.CSSProperties = {
  flexShrink: 0,
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '16px 24px', background: '#fff',
  borderBottom: '1px solid #f0f0f0',
}

export default TopicManagePage
