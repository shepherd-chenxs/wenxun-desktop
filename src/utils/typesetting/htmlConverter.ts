// Markdown 转带内联样式的 HTML (用于微信公众号发布)

import { marked } from 'marked'
import { getTemplateById } from './templates'

// 配置 marked
marked.setOptions({
  breaks: true,
  gfm: true
})

interface InlineStyleConfig {
  h1?: React.CSSProperties
  h2?: React.CSSProperties
  h3?: React.CSSProperties
  p?: React.CSSProperties
  ul?: React.CSSProperties
  ol?: React.CSSProperties
  li?: React.CSSProperties
}

// 获取模板的内联样式配置
const getTemplateStyles = (templateId: string): InlineStyleConfig => {
  const styles: Record<string, InlineStyleConfig> = {
    default: {
      h1: { fontSize: '28px', fontWeight: 'bold', marginBottom: '20px', color: '#333' },
      h2: { fontSize: '24px', fontWeight: 'bold', marginBottom: '16px', color: '#333' },
      h3: { fontSize: '20px', fontWeight: 'bold', marginBottom: '14px', color: '#333' },
      p: { lineHeight: '1.8', marginBottom: '16px', textIndent: '2em' },
      ul: { marginBottom: '16px', paddingLeft: '2em' },
      ol: { marginBottom: '16px', paddingLeft: '2em' },
      li: { marginBottom: '8px', lineHeight: '1.6' }
    },
    simple: {
      h1: { fontSize: '30px', fontWeight: 'bold', marginBottom: '18px', color: '#333' },
      h2: { fontSize: '26px', fontWeight: 'bold', marginBottom: '14px', color: '#333' },
      h3: { fontSize: '22px', fontWeight: 'bold', marginBottom: '12px', color: '#333' },
      p: { lineHeight: '1.7', marginBottom: '14px', textIndent: '0' },
      ul: { marginBottom: '14px', paddingLeft: '1.5em' },
      ol: { marginBottom: '14px', paddingLeft: '1.5em' },
      li: { marginBottom: '6px', lineHeight: '1.5' }
    },
    professional: {
      h1: { 
        fontSize: '32px', 
        fontWeight: 'bold', 
        marginBottom: '24px', 
        color: '#222',
        borderBottom: '2px solid #e8e8e8',
        paddingBottom: '8px'
      },
      h2: { 
        fontSize: '26px', 
        fontWeight: 'bold', 
        marginBottom: '20px', 
        color: '#222',
        borderBottom: '1px solid #e8e8e8',
        paddingBottom: '6px'
      },
      h3: { fontSize: '22px', fontWeight: 'bold', marginBottom: '16px', color: '#222' },
      p: { lineHeight: '1.8', marginBottom: '18px', textIndent: '2em' },
      ul: { marginBottom: '18px', paddingLeft: '2em' },
      ol: { marginBottom: '18px', paddingLeft: '2em' },
      li: { marginBottom: '8px', lineHeight: '1.6' }
    }
  }
  
  return styles[templateId] || styles.default
}

// 将 CSS 属性对象转换为内联样式字符串
const styleToString = (style: React.CSSProperties): string => {
  return Object.entries(style)
    .map(([key, value]) => {
      const cssKey = key.replace(/([A-Z])/g, '-$1').toLowerCase()
      return `${cssKey}: ${value}`
    })
    .join('; ')
}

// 为 HTML 元素添加内联样式
const addInlineStyles = (html: string, styles: InlineStyleConfig): string => {
  let styledHtml = html
  
  // 为标题添加样式
  if (styles.h1) {
    const styleStr = styleToString(styles.h1)
    styledHtml = styledHtml.replace(/<h1([^>]*)>/g, `<h1$1 style="${styleStr}">`)
  }
  if (styles.h2) {
    const styleStr = styleToString(styles.h2)
    styledHtml = styledHtml.replace(/<h2([^>]*)>/g, `<h2$1 style="${styleStr}">`)
  }
  if (styles.h3) {
    const styleStr = styleToString(styles.h3)
    styledHtml = styledHtml.replace(/<h3([^>]*)>/g, `<h3$1 style="${styleStr}">`)
  }
  
  // 为段落添加样式
  if (styles.p) {
    const styleStr = styleToString(styles.p)
    styledHtml = styledHtml.replace(/<p([^>]*)>/g, `<p$1 style="${styleStr}">`)
  }
  
  // 为列表添加样式
  if (styles.ul) {
    const styleStr = styleToString(styles.ul)
    styledHtml = styledHtml.replace(/<ul([^>]*)>/g, `<ul$1 style="${styleStr}">`)
  }
  if (styles.ol) {
    const styleStr = styleToString(styles.ol)
    styledHtml = styledHtml.replace(/<ol([^>]*)>/g, `<ol$1 style="${styleStr}">`)
  }
  if (styles.li) {
    const styleStr = styleToString(styles.li)
    styledHtml = styledHtml.replace(/<li([^>]*)>/g, `<li$1 style="${styleStr}">`)
  }
  
  return styledHtml
}

/**
 * 将 Markdown 转换为带内联样式的 HTML
 * @param markdown Markdown 内容
 * @param templateId 模板 ID
 * @returns 带内联样式的 HTML 字符串
 */
export const markdownToStyledHtml = (markdown: string, templateId: string = 'default'): string => {
  try {
    // 验证模板
    const template = getTemplateById(templateId)
    if (!template) {
      console.warn(`Template "${templateId}" not found, using default`)
    }
    
    // 1. 将 Markdown 转换为 HTML
    const html = marked(markdown) as string
    
    // 2. 获取模板的内联样式配置
    const styles = getTemplateStyles(templateId)
    
    // 3. 为 HTML 元素添加内联样式
    const styledHtml = addInlineStyles(html, styles)
    
    return styledHtml
  } catch (error) {
    console.error('Markdown to Styled HTML conversion error:', error)
    // 降级处理:返回原始 Markdown 转换的 HTML(无样式)
    return marked(markdown) as string
  }
}
