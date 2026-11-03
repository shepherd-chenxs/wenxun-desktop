// 排版处理引擎 (已废弃 - 改用 CSS 样式隔离方案)
// 此文件保留仅为向后兼容,新功能请使用 CSS 类名方式

interface Template {
  id: string
  name: string
  description: string
  rules: {
    paragraphSpacing: boolean
    headingOptimization: boolean
    listOptimization: boolean
    textIndent: boolean
    headingSpace: boolean
  }
}

/**
 * @deprecated 此函数已废弃,不再修改 Markdown 内容
 * 现在采用 CSS 样式隔离方案,通过模板类名控制样式
 * 请直接使用模板 ID 作为 CSS 类名: `editor-template-${templateId}`
 */
export const processTypesetting = (content: string, _template: Template): string => {
  console.warn('processTypesetting is deprecated. Use CSS class names instead.')
  // 返回原始内容,不做任何修改
  return content
}