import { FC } from 'react'
import { Drawer, Radio, Button, Space } from 'antd'
import MDEditor from '@uiw/react-md-editor'
import '@uiw/react-md-editor/markdown-editor.css'
import '@uiw/react-markdown-preview/markdown.css'

interface Template {
  id: string
  name: string
  description: string
  color: string
  icon: string
  rules: {
    paragraphSpacing: boolean
    headingOptimization: boolean
    listOptimization: boolean
    textIndent: boolean
    headingSpace: boolean
  }
}

interface SmartTypesettingModalProps {
  visible: boolean
  templates: Template[]
  selectedTemplate: string
  originalContent: string
  onTemplateChange: (templateId: string) => void
  onApply: () => void
  onCancel: () => void
}

const SmartTypesettingModal: FC<SmartTypesettingModalProps> = ({
  visible,
  templates,
  selectedTemplate,
  originalContent,
  onTemplateChange,
  onApply,
  onCancel
}) => {
  return (
    <Drawer
      title="智能排版"
      placement="right"
      width={1000}
      open={visible}
      onClose={onCancel}
      footer={
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <Button onClick={onCancel}>取消</Button>
          <Button type="primary" onClick={onApply}>应用排版</Button>
        </div>
      }
    >
      <div style={{ display: 'flex', gap: 24, height: '100%' }}>
        {/* 左侧：模板选择 */}
        <div style={{ width: 300, flexShrink: 0 }}>
          <h3 style={{ marginBottom: 16 }}>选择模板</h3>
          <Radio.Group
            value={selectedTemplate}
            onChange={(e) => onTemplateChange(e.target.value)}
            style={{ width: '100%' }}
          >
            {templates.map((template) => (
              <Radio
                key={template.id}
                value={template.id}
                style={{ 
                  display: 'block', 
                  marginBottom: 12,
                  padding: 16,
                  border: selectedTemplate === template.id ? `2px solid ${template.color}` : '1px solid #f0f0f0',
                  borderRadius: 8,
                  transition: 'all 0.3s',
                  backgroundColor: selectedTemplate === template.id ? `${template.color}15` : '#fff',
                  boxShadow: selectedTemplate === template.id ? `0 2px 8px ${template.color}30` : 'none'
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', marginBottom: 8 }}>
                  <span style={{ 
                    fontSize: 20, 
                    marginRight: 8,
                    filter: selectedTemplate === template.id ? 'none' : 'grayscale(100%)'
                  }}>
                    {template.icon}
                  </span>
                  <span style={{ 
                    fontWeight: 'bold',
                    color: selectedTemplate === template.id ? template.color : '#333',
                    fontSize: 15
                  }}>
                    {template.name}
                  </span>
                </div>
                <div style={{ color: '#666', lineHeight: 1.5, fontSize: 12 }}>
                  {template.description}
                </div>
              </Radio>
            ))}
          </Radio.Group>
        </div>

        {/* 右侧：预览区域 */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
            <h3 style={{ margin: 0 }}>预览效果</h3>
            <Space>
              <span style={{ color: '#666', fontSize: 12 }}>当前模板：{templates.find(t => t.id === selectedTemplate)?.name}</span>
            </Space>
          </div>
          
          {/* 预览内容 - 使用原始 Markdown + CSS 类名 */}
          <div 
            style={{ 
              flex: 1,
              overflow: 'auto',
              borderRadius: 4,
              border: '1px solid #e8e8e8'
            }}
            className={`preview-template-${selectedTemplate}`}
          >
            <MDEditor.Markdown 
              source={originalContent}
              style={{ 
                minHeight: '100%',
                padding: 24
              }}
            />
          </div>
        </div>
      </div>
    </Drawer>
  )
}

export default SmartTypesettingModal