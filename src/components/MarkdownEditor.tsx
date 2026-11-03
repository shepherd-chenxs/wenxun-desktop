import { useState, useEffect, memo, useCallback, useRef } from 'react'
import MDEditor from '@uiw/react-md-editor'
import '@uiw/react-md-editor/markdown-editor.css'
import '@uiw/react-markdown-preview/markdown.css'

interface MarkdownEditorProps {
  content: string
  onChange: (content: string) => void
  templateClass?: string
}

const MarkdownEditor = memo(({ content, onChange, templateClass }: MarkdownEditorProps) => {
  const [mdContent, setMdContent] = useState(content)
  const [exitFullscreenKey, setExitFullscreenKey] = useState(0)
  const isFullscreenRef = useRef(false)

  useEffect(() => {
    if (content !== mdContent) {
      setMdContent(content)
    }
  }, [content, mdContent])

  // 监听全屏状态变化
  const handleFullscreenChange = useCallback(() => {
    isFullscreenRef.current = !!document.fullscreenElement
    
    // 只在退出全屏时重新挂载组件，修复全屏按钮
    if (!isFullscreenRef.current) {
      requestAnimationFrame(() => {
        setExitFullscreenKey(prev => prev + 1)
      })
    }
  }, [])

  useEffect(() => {
    document.addEventListener('fullscreenchange', handleFullscreenChange)
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange)
    }
  }, [handleFullscreenChange])

  const handleChange = (value?: string) => {
    if (value !== undefined) {
      setMdContent(value)
      onChange(value)
    }
  }

  return (
    <div className={`markdown-editor-container ${templateClass || ''}`} style={{ height: '100%' }}>
      <MDEditor
        key={exitFullscreenKey}
        value={mdContent}
        onChange={handleChange}
        height="100%"
        previewOptions={{
          wrapperElement: {
            'data-color-mode': 'light'
          }
        }}
        className="editor-markdown-editor"
      />
    </div>
  )
})

MarkdownEditor.displayName = 'MarkdownEditor'

export default MarkdownEditor