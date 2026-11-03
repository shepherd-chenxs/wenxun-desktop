/// <reference types="vite/client" />

declare global {
  interface Window {
    /** Tauri 2 withGlobalTauri 注入的全局对象（浏览器/测试环境不存在） */
    __TAURI__?: unknown
  }
}

export {}
