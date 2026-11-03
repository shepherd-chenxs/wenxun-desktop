import { useState, useEffect } from 'react'
import { Button, Form, Input, InputNumber, message, Popconfirm, Select, Space, Tag, Tree, Typography } from 'antd'
import { CopyOutlined, DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import type { VirtualBlogger } from '../services/storeService'
import { storeService } from '../services/storeService'

const { TextArea } = Input
const { Title, Text, Paragraph } = Typography

const SIDER_WIDTH = 280

const VirtualBloggerPage = () => {
  const [bloggers, setBloggers] = useState<VirtualBlogger[]>([])
  const [selectedBlogger, setSelectedBlogger] = useState<VirtualBlogger | null>(null)
  const [isEditing, setIsEditing] = useState(false)
  const [formMode, setFormMode] = useState<'create' | 'edit'>('create')
  const [expandedKeys, setExpandedKeys] = useState<React.Key[]>(['builtin', 'custom'])
  const [form] = Form.useForm()

  const builtin = bloggers.filter((b) => b.is_builtin)
  const custom = bloggers.filter((b) => !b.is_builtin)

  useEffect(() => {
    void loadBloggers()
  }, [])

  useEffect(() => {
    if (bloggers.length > 0 && !selectedBlogger) {
      setSelectedBlogger(bloggers.find((b) => b.is_builtin) || bloggers[0])
    }
  }, [bloggers])

  useEffect(() => {
    if (selectedBlogger && isEditing) {
      form.setFieldsValue(selectedBlogger)
    }
  }, [selectedBlogger, isEditing])

  const loadBloggers = async () => {
    try {
      setBloggers(await storeService.getVirtualBloggers())
    } catch (error) {
      console.error('加载写作人设失败:', error)
      message.error('加载写作人设失败')
    }
  }

  const handleAdd = () => {
    setSelectedBlogger(null)
    setFormMode('create')
    setIsEditing(true)
    form.resetFields()
    form.setFieldsValue({ default_word_count: 1500 })
  }

  const handleCancel = () => {
    setIsEditing(false)
    form.resetFields()
  }

  const handleDelete = async (id: string) => {
    try {
      await storeService.deleteVirtualBlogger(id)
      await loadBloggers()
      if (selectedBlogger?.id === id) setSelectedBlogger(null)
      message.success('删除成功')
    } catch (error) {
      message.error('删除失败')
    }
  }

  const handleCopyBuiltin = async (b: VirtualBlogger) => {
    try {
      const copy: VirtualBlogger = {
        ...b,
        id: Date.now().toString(),
        name: `${b.name}·副本`,
        is_builtin: false,
        preset_key: '',
      }
      await storeService.saveVirtualBlogger(copy)
      await loadBloggers()
      setSelectedBlogger(copy)
      setFormMode('edit')
      setIsEditing(true)
      form.setFieldsValue(copy)
      message.success('已创建自定义人设副本，可继续修改后保存')
    } catch (e) {
      message.error((e as Error).message)
    }
  }

  const handleSubmit = async (values: any) => {
    try {
      const clean = { ...values, topic_category: '', topic_category_id: '', topic_category_name: '', is_builtin: false, preset_key: '', lessons: selectedBlogger?.lessons || '' }
      if (selectedBlogger && isEditing && !selectedBlogger.is_builtin) {
        const updatedBlogger = { ...selectedBlogger, ...clean }
        await storeService.saveVirtualBlogger(updatedBlogger)
        setSelectedBlogger(updatedBlogger)
        message.success('更新成功')
      } else {
        const newBlogger = { id: Date.now().toString(), ...clean, default_word_count: values.default_word_count || 1500 }
        await storeService.saveVirtualBlogger(newBlogger)
        setSelectedBlogger(newBlogger)
        message.success('人设已创建。提示：也可以在爆文工坊里从拆解报告一键沉淀人设')
      }
      await loadBloggers()
      setIsEditing(false)
    } catch (error) {
      message.error('保存失败')
    }
  }

  const renderBloggerDetail = (b: VirtualBlogger) => (
    <>
      <div style={contentHeaderStyle}>
        <Title level={4} style={{ margin: 0 }}>
          {b.name} {b.is_builtin && <Tag color="blue">内置写手</Tag>}
        </Title>
        <Space>
          {b.is_builtin ? (
            <Button type="primary" ghost icon={<CopyOutlined />} onClick={() => void handleCopyBuiltin(b)}>
              以此写手为模板创建自定义
            </Button>
          ) : (
            <>
              <Button icon={<EditOutlined />} onClick={() => { setFormMode('edit'); setIsEditing(true) }}>编辑</Button>
              <Popconfirm title="确定删除此人设？" onConfirm={() => void handleDelete(b.id)} okText="确定" cancelText="取消">
                <Button danger icon={<DeleteOutlined />}>删除</Button>
              </Popconfirm>
            </>
          )}
        </Space>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        {b.is_builtin && (
          <Paragraph type="secondary" style={{ fontSize: 12 }}>
            内置写手由系统预置完整写作方法论，定时任务与热点雷达可直接使用；内置写手不可修改，如需个性化请点击右上角「以此写手为模板创建自定义」。
          </Paragraph>
        )}
        <Paragraph><Text strong>人设描述：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.persona}</div></Paragraph>
        <Paragraph><Text strong>选题与表达要领：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.writing_style}</div></Paragraph>
        <Paragraph><Text strong>语言风格：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.language_style}</div></Paragraph>
        <Paragraph><Text strong>文章结构：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.article_structure}</div></Paragraph>
        <Paragraph><Text strong>排版规范：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.layout_format}</div></Paragraph>
        <Paragraph><Text strong>语气关键词：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.tone_keywords}</div></Paragraph>
        <Paragraph><Text strong>内容红线：</Text><div style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>{b.content_bottom_line}</div></Paragraph>
        {!b.is_builtin && (
          <Paragraph>
            <Space align="baseline">
              <Text strong>体检教训清单：</Text>
              {b.lessons && (
                <Popconfirm title="清空后不可恢复，确定？" onConfirm={async () => {
                  const updated = { ...b, lessons: '' }
                  await storeService.saveVirtualBlogger(updated)
                  setSelectedBlogger(updated)
                  await loadBloggers()
                  message.success('已清空教训清单')
                }} okText="确定" cancelText="取消">
                  <Button size="small" type="text" danger>清空</Button>
                </Popconfirm>
              )}
            </Space>
            <div style={{ marginTop: 8, whiteSpace: 'pre-wrap', fontSize: 13 }}>
              {b.lessons || <Text type="secondary">暂无。在编辑器「体检报告」里点「记入人设」可积累，生成时自动规避。</Text>}
            </div>
          </Paragraph>
        )}
        <Paragraph><Text strong>默认字数：</Text><Text>{b.default_word_count}</Text></Paragraph>
      </div>
    </>
  )

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#f0f2f5' }}>
      <style>{`
        .writer-tree .ant-tree-switcher { width: 20px; margin-inline-end: 0; }
        .writer-tree .ant-tree-node-content-wrapper { padding-inline: 0 4px; }
      `}</style>
      <div style={{ flexShrink: 0, padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>写作人设</Title>
      </div>
      <div style={{ flexShrink: 0, height: 16, background: '#f0f2f5' }} />
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden', padding: '0 16px 16px 16px', gap: 16 }}>
        {/* 左侧边栏：二级菜单树 = 内置写手 / 我的写手 */}
        <div style={{ width: SIDER_WIDTH, flexShrink: 0, background: '#fff', borderRadius: 8, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <div style={siderHeaderStyle}>
            <Title level={5} style={{ margin: 0 }}>人设列表</Title>
            <Button type="primary" size="small" icon={<PlusOutlined />} onClick={handleAdd}>创建</Button>
          </div>
          <div style={{ flex: 1, overflowY: 'auto', paddingBottom: 8 }}>
            <Tree
              className="writer-tree"
              blockNode
              expandedKeys={expandedKeys}
              onExpand={(keys) => setExpandedKeys(keys)}
              selectedKeys={selectedBlogger ? [selectedBlogger.id] : []}
              onSelect={(keys) => {
                const id = keys[0]
                const blogger = bloggers.find((b) => b.id === id)
                if (blogger) {
                  setSelectedBlogger(blogger)
                  setIsEditing(false)
                }
              }}
              treeData={[
                {
                  key: 'builtin',
                  title: '内置写手',
                  children: builtin.map((b) => ({
                    key: b.id,
                    title: b.name,
                  })),
                },
                {
                  key: 'custom',
                  title: '我的写手',
                  children: custom.map((b) => ({
                    key: b.id,
                    title: b.name,
                  })),
                },
              ]}
            />
          </div>
        </div>

        {/* 右侧内容 */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', background: '#fff', borderRadius: 8 }}>
          {isEditing ? (
            <>
              <div style={contentHeaderStyle}>
                <Title level={4} style={{ margin: 0 }}>
                  {formMode === 'edit' ? '编辑自定义人设' : '新建自定义人设'}
                </Title>
                <Space>
                  <Button type="primary" onClick={() => form.submit()}>保存</Button>
                  <Button onClick={handleCancel}>取消</Button>
                </Space>
              </div>
              <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
                <div style={{ marginBottom: 16 }}>
                  <Text strong style={{ marginRight: 8, fontSize: 13 }}>赛道模板（可选）：</Text>
                  <Select
                    placeholder="选择内置写手自动填充下方表单，再按需修改"
                    style={{ width: 320 }}
                    options={builtin.map((b) => ({ value: b.id, label: b.name }))}
                    value={undefined}
                    onChange={(id) => {
                      const tpl = builtin.find((b) => b.id === id)
                      if (tpl) {
                        form.setFieldsValue({
                          name: `${tpl.name}·自定义`,
                          persona: tpl.persona,
                          writing_style: tpl.writing_style,
                          language_style: tpl.language_style,
                          article_structure: tpl.article_structure,
                          layout_format: tpl.layout_format,
                          tone_keywords: tpl.tone_keywords,
                          content_bottom_line: tpl.content_bottom_line,
                          default_word_count: tpl.default_word_count,
                        })
                        message.info(`已按「${tpl.name}」填充表单，请修改后保存`)
                      }
                    }}
                  />
                </div>
                <Paragraph type="secondary" style={{ fontSize: 12 }}>
                  也可以完全手写；或在爆文工坊里从拆解报告一键「存为写作人设」。
                </Paragraph>
                <Form form={form} layout="vertical" onFinish={handleSubmit} style={{ maxWidth: 800 }}>
                  <Form.Item label="人设名称" name="name" rules={[{ required: true, message: '请输入人设名称' }]}>
                    <Input placeholder="例如：深夜情感观察者" />
                  </Form.Item>
                  <Form.Item label="人设描述" name="persona" rules={[{ required: true, message: '请输入人设描述' }]}>
                    <TextArea placeholder="这个号是谁、写什么、读者为什么信他" rows={3} />
                  </Form.Item>
                  <Form.Item label="选题与表达要领" name="writing_style" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="选题定位、受众画像、钩子库（可直接套用的开头句式）、表达要领" rows={5} />
                  </Form.Item>
                  <Form.Item label="语言风格" name="language_style" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="句式、人称、段落节奏、金句密度" rows={3} />
                  </Form.Item>
                  <Form.Item label="文章结构" name="article_structure" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="从开头到结尾的分段公式" rows={4} />
                  </Form.Item>
                  <Form.Item label="排版规范" name="layout_format" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="篇幅、段落长度、小标题、列表习惯" rows={2} />
                  </Form.Item>
                  <Form.Item label="语气关键词" name="tone_keywords" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="3-5 个，逗号分隔" rows={1} />
                  </Form.Item>
                  <Form.Item label="内容红线" name="content_bottom_line" rules={[{ required: true, message: '请输入' }]}>
                    <TextArea placeholder="不触碰的内容边界" rows={2} />
                  </Form.Item>
                  <Form.Item label="默认字数" name="default_word_count" rules={[{ required: true, message: '请输入' }]}>
                    <InputNumber min={500} max={5000} style={{ width: '100%' }} />
                  </Form.Item>
                </Form>
              </div>
            </>
          ) : selectedBlogger ? (
            renderBloggerDetail(selectedBlogger)
          ) : (
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <Text type="secondary">选择左侧人设查看详情</Text>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

const siderHeaderStyle: React.CSSProperties = {
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '16px', borderBottom: '1px solid #f0f0f0',
}

const contentHeaderStyle: React.CSSProperties = {
  flexShrink: 0,
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '16px 24px', background: '#fff',
  borderBottom: '1px solid #f0f0f0',
}

export default VirtualBloggerPage
