/**
 * 从任意错误形态中提取可展示的文案，避免空 message 导致 toast 只显示一个 ×。
 * 兼容：Error、字符串、Tauri invoke 返回的对象、axios/内部嵌套错误。
 */
export function getErrorMessage(e: unknown, fallback = '操作失败，请稍后重试'): string {
  if (typeof e === 'string') return e.trim() || fallback
  if (e instanceof Error) return e.message.trim() || fallback

  if (e && typeof e === 'object') {
    const obj = e as Record<string, unknown>
    const candidates = [obj.message, obj.error, obj.msg, obj.description, obj.detail, obj.reason]
    for (const c of candidates) {
      if (typeof c === 'string' && c.trim()) return c.trim()
      if (c && typeof c === 'object') {
        const inner = c as Record<string, unknown>
        for (const m of [inner.message, inner.error, inner.msg, inner.description]) {
          if (typeof m === 'string' && m.trim()) return m.trim()
        }
      }
    }
    try {
      const text = JSON.stringify(e)
      if (text && text !== '{}' && text !== '[]') return text
    } catch {
      // ignore
    }
  }

  return fallback
}
