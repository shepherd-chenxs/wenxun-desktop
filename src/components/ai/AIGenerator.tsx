import { useState, useEffect } from 'react'
import { Input, Button, Select, message, Modal, Form, InputNumber } from 'antd'
import { SendOutlined } from '@ant-design/icons'
import type { TopicCategory, VirtualBlogger } from '../../services/storeService'
import { storeService } from '../../services/storeService'

interface AIGeneratorProps {
  onGenerate: (title: string, content: string) => void
}

const { Option } = Select
const { TextArea } = Input

const AIGenerator: React.FC<AIGeneratorProps> = ({ onGenerate }) => {
  const [modalVisible, setModalVisible] = useState(false)
  const [loading, setLoading] = useState(false)
  const [topicCategories, setTopicCategories] = useState<TopicCategory[]>([])
  const [virtualBloggers, setVirtualBloggers] = useState<VirtualBlogger[]>([])
  const [selectedTopicCategory, setSelectedTopicCategory] = useState<string | undefined>()
  const [form] = Form.useForm()

  useEffect(() => {
    const loadData = async () => {
      await loadTopicCategories()
      await loadVirtualBloggers()
    }
    loadData()
  }, [])

  const loadTopicCategories = async () => {
    try {
      const categories = await storeService.getTopicCategories()
      setTopicCategories(categories)
    } catch (error) {
      console.error('加载题材分类失败:', error)
      message.error('加载题材分类失败')
      const defaultCategories: TopicCategory[] = [
        {
          id: '1',
          name: '历史',
          description: '历史事件、人物传记、历史文化等内容',
          created_at: new Date().toISOString(),
        },
        {
          id: '2',
          name: '搞钱',
          description: '赚钱方法、理财投资、副业创业',
          created_at: new Date().toISOString(),
        },
        {
          id: '3',
          name: '情感',
          description: '情感关系、心理成长、人际交往',
          created_at: new Date().toISOString(),
        },
        {
          id: '4',
          name: '认知',
          description: '思维模式、认知提升、学习方法',
          created_at: new Date().toISOString(),
        },
        {
          id: '5',
          name: '工具',
          description: '效率工具、软件推荐、技术应用',
          created_at: new Date().toISOString(),
        },
        {
          id: '6',
          name: '世相',
          description: '社会现象、热点评论、生活观察',
          created_at: new Date().toISOString(),
        },
      ]
      setTopicCategories(defaultCategories)
    }
  }

  const loadVirtualBloggers = async () => {
    try {
      const bloggers = await storeService.getVirtualBloggers()
      setVirtualBloggers(bloggers)
    } catch (error) {
      console.error('加载虚拟博主失败:', error)
      message.error('加载虚拟博主失败')
      setVirtualBloggers([])
    }
  }

  const handleOpenModal = () => {
    if (form) {
      form.resetFields()
      form.setFieldsValue({
        wordCount: 800,
      })
    }
    setSelectedTopicCategory(undefined)
    setModalVisible(true)
  }

  const handleTopicCategoryChange = (value: string) => {
    setSelectedTopicCategory(value)
    if (form) {
      form.setFieldsValue({ virtualBlogger: undefined })
    }
  }

  const handleVirtualBloggerChange = (value: string) => {
    if (form) {
      const selectedBlogger = virtualBloggers.find(b => b.id === value)
      if (selectedBlogger) {
        form.setFieldsValue({ wordCount: selectedBlogger.default_word_count })
      }
    }
  }

  const handleGenerate = async (values: {
    topic: string;
    wordCount: number;
    topicCategory?: string;
    virtualBlogger?: string
  }) => {
    setLoading(true)
    try {
      let selectedBlogger: VirtualBlogger | undefined
      if (values.virtualBlogger) {
        selectedBlogger = virtualBloggers.find(b => b.id === values.virtualBlogger)
      }

      let prompt = values.topic
      if (selectedBlogger) {
        prompt = `你是${selectedBlogger.name}，${selectedBlogger.persona}\n` +
                `文风风格：${selectedBlogger.writing_style}\n` +
                `语言风格：${selectedBlogger.language_style}\n` +
                `文章结构：${selectedBlogger.article_structure}\n` +
                `排版格式：${selectedBlogger.layout_format}\n` +
                `语气关键词：${selectedBlogger.tone_keywords}\n` +
                `内容底线：${selectedBlogger.content_bottom_line}\n` +
                `请根据以上设定，为"${values.topic}"这个主题生成一篇文章。`
      }
      console.log('发送给AI的prompt:', prompt)

      const titleResponse = await storeService.generateArticle(
        `请为"${values.topic}"这个主题生成一个吸引人的文章标题，只需要返回标题，不要其他内容`,
        values.wordCount,
        selectedBlogger?.writing_style || '正式'
      )
      const title = titleResponse.trim().replace(/["'"]/g, '')

      const content = await storeService.generateArticle(
        prompt,
        values.wordCount,
        selectedBlogger?.writing_style || '正式'
      )

      onGenerate(title, content)
      message.success('文章生成成功')

      const history = {
        id: Date.now().toString(),
        topic: values.topic,
        content,
        created_at: new Date().toISOString()
      }
      await storeService.saveGenerationHistory(history)

      setModalVisible(false)
    } catch (error) {
      console.error('文章生成失败:', error)
      message.error(`文章生成失败: ${error instanceof Error ? error.message : String(error)}`)
    } finally {
      setLoading(false)
    }
  }

  const getFilteredBloggers = (topicCategoryId?: string) => {
    if (!topicCategoryId) return virtualBloggers
    return virtualBloggers.filter(blogger => blogger.topic_category_id === topicCategoryId)
  }

  return (
    <>
      <Button type="primary" icon={<SendOutlined />} onClick={handleOpenModal}>
        生成文章
      </Button>

      <Modal
        title="AI生成文章"
        open={modalVisible}
        onCancel={() => setModalVisible(false)}
        footer={null}
        width={600}
      >
        <Form form={form} layout="vertical" onFinish={handleGenerate}>
          <Form.Item
            label="文章主题"
            name="topic"
            rules={[{ required: true, message: '请输入文章主题' }]}
          >
            <TextArea
              placeholder="请输入想要生成的文章主题或关键词"
              rows={3}
            />
          </Form.Item>

          <Form.Item
            label="题材分类"
            name="topicCategory"
          >
            <Select
              placeholder="请选择题材分类"
              onChange={handleTopicCategoryChange}
            >
              {topicCategories.map((category) => (
                <Option key={category.id} value={category.id}>
                  {category.name}
                </Option>
              ))}
            </Select>
          </Form.Item>

          <Form.Item
            label="虚拟博主"
            name="virtualBlogger"
          >
            <Select
              placeholder="请选择虚拟博主"
              onChange={handleVirtualBloggerChange}
            >
              {selectedTopicCategory && getFilteredBloggers(selectedTopicCategory).map((blogger) => (
                <Option key={blogger.id} value={blogger.id}>
                  {blogger.name}
                </Option>
              ))}
            </Select>
          </Form.Item>

          <Form.Item
            label="文章字数"
            name="wordCount"
            rules={[{ required: true, message: '请输入文章字数' }]}
          >
            <InputNumber
              min={300}
              max={3000}
              step={100}
              style={{ width: '100%' }}
              placeholder="例如：800"
            />
          </Form.Item>

          <Form.Item>
            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
              <Button onClick={() => setModalVisible(false)}>取消</Button>
              <Button type="primary" htmlType="submit" loading={loading}>
                生成文章
              </Button>
            </div>
          </Form.Item>
        </Form>
      </Modal>
    </>
  )
}

export default AIGenerator