/**
 * 输入验证工具类
 * 用于验证用户输入，防止XSS攻击和注入问题
 */

/**
 * 验证文章标题
 * @param title 文章标题
 * @returns 验证结果，包含是否有效和错误信息
 */
export const validateTitle = (title: string): { valid: boolean; error?: string } => {
  if (!title || title.trim().length === 0) {
    return { valid: false, error: '标题不能为空' };
  }
  if (title.length > 200) {
    return { valid: false, error: '标题长度不能超过200个字符' };
  }
  // 检查是否包含危险字符，防止XSS攻击
  const dangerousPatterns = /[<>&"']/;
  if (dangerousPatterns.test(title)) {
    return { valid: false, error: '标题包含非法字符' };
  }
  return { valid: true };
};

/**
 * 验证文章内容
 * @param content 文章内容
 * @returns 验证结果，包含是否有效和错误信息
 */
export const validateContent = (content: string): { valid: boolean; error?: string } => {
  if (!content || content.trim().length === 0) {
    return { valid: false, error: '内容不能为空' };
  }
  if (content.length > 50000) {
    return { valid: false, error: '内容长度不能超过50000个字符' };
  }
  return { valid: true };
};

/**
 * 验证AI生成参数
 * @param topic 文章主题
 * @param style 文章风格
 * @param wordCount 字数
 * @returns 验证结果，包含是否有效和错误信息
 */
export const validateAIGenerationParams = (topic: string, style: string, wordCount: number): { valid: boolean; error?: string } => {
  if (!topic || topic.trim().length === 0) {
    return { valid: false, error: '文章主题不能为空' };
  }
  if (!style || style.trim().length === 0) {
    return { valid: false, error: '文章风格不能为空' };
  }
  if (!wordCount || wordCount < 100 || wordCount > 10000) {
    return { valid: false, error: '字数必须在100-10000之间' };
  }
  return { valid: true };
};

/**
 * 验证微信公众号配置
 * @param appId 公众号AppID
 * @param appSecret 公众号AppSecret
 * @returns 验证结果，包含是否有效和错误信息
 */
export const validateWechatConfig = (appId: string, appSecret: string): { valid: boolean; error?: string } => {
  if (!appId || appId.trim().length === 0) {
    return { valid: false, error: 'AppID不能为空' };
  }
  if (!appSecret || appSecret.trim().length === 0) {
    return { valid: false, error: 'AppSecret不能为空' };
  }
  return { valid: true };
};

/**
 * 转义HTML特殊字符，防止XSS攻击
 * @param text 要转义的文本
 * @returns 转义后的文本
 */
export const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
};
