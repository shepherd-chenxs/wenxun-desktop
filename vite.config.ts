import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [
    react(),
  ],
  base: './',
  server: {
    port: 5174,
  },
  build: {
    rollupOptions: {
      external: ['@tauri-apps/api'],
      output: {
        // 按库拆包：并行加载 + 利用缓存，显著缩短首屏 JS 解析时间
        manualChunks(id) {
          if (!id.includes('node_modules')) return
          if (/[\/]node_modules[\/](antd|@ant-design)[\/]/.test(id)) return 'antd'
          if (/[\/]node_modules[\/](@tiptap|@uiw|marked|prosemirror)/.test(id)) return 'editor'
          if (/[\/]node_modules[\/](react|react-dom|react-router|scheduler|zustand|@remix-run)/.test(id)) return 'react'
          if (/[\/]node_modules[\/](axios|crypto-js|dayjs|sql.js|keytar|form-data|formik|yup|inversify|reflect-metadata)/.test(id)) return 'misc'
        },
      },
    },
  },
})
