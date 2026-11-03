// 模板配置系统

export interface TypesettingTemplate {
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

// 内置模板
export const templates: TypesettingTemplate[] = [
  {
    id: 'default',
    name: '默认模板',
    description: '标准排版格式，包含段落空行、标题优化、列表优化、首行缩进',
    color: '#1890ff',
    icon: '📝',
    rules: {
      paragraphSpacing: true,
      headingOptimization: true,
      listOptimization: true,
      textIndent: true,
      headingSpace: false
    }
  },
  {
    id: 'simple',
    name: '简洁模板',
    description: '简洁排版格式，包含段落空行、标题优化、列表优化，无首行缩进',
    color: '#52c41a',
    icon: '✨',
    rules: {
      paragraphSpacing: true,
      headingOptimization: true,
      listOptimization: true,
      textIndent: false,
      headingSpace: false
    }
  },
  {
    id: 'professional',
    name: '专业模板',
    description: '专业排版格式，包含段落空行、标题优化（带空行）、列表优化、首行缩进',
    color: '#722ed1',
    icon: '🎨',
    rules: {
      paragraphSpacing: true,
      headingOptimization: true,
      listOptimization: true,
      textIndent: true,
      headingSpace: true
    }
  }
]

// 根据ID获取模板
export const getTemplateById = (id: string): TypesettingTemplate | undefined => {
  return templates.find(template => template.id === id)
}

// 获取所有模板
export const getAllTemplates = (): TypesettingTemplate[] => {
  return templates
}