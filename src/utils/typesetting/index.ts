// 排版工具主入口
import { getAllTemplates, getTemplateById } from './templates'
import { processTypesetting } from './processor'
import { markdownToStyledHtml } from './htmlConverter'

export {
  getAllTemplates,
  getTemplateById,
  processTypesetting,
  markdownToStyledHtml
}