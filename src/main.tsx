import React from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import App from './App'
import 'antd/dist/reset.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)

// 页面加载完成后显示窗口；若 Rust 侧 on_page_load 未及时显示，这里做兜底
const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined
if (isTauri) {
  const showWindow = () => getCurrentWindow().show().catch(() => {})
  if (document.readyState === 'complete') {
    showWindow()
  } else {
    window.addEventListener('load', showWindow)
  }
}
