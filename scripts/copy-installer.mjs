/**
 * 打包后处理：把 NSIS 安装包从 src-tauri/target 深处拷到根目录 release/。
 * 在 tauri:build 的 tauri build 之后自动执行。
 */
import { copyFileSync, readdirSync, mkdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const srcDir = 'src-tauri/target/release/bundle/nsis'
const outDir = 'release'
mkdirSync(outDir, { recursive: true })

const installers = readdirSync(srcDir)
  .filter((f) => f.endsWith('-setup.exe'))
  .map((f) => ({ name: f, mtime: statSync(join(srcDir, f)).mtimeMs }))
  .sort((a, b) => b.mtime - a.mtime)

if (installers.length === 0) {
  console.error('[copy-installer] 未找到安装包，跳过拷贝')
  process.exit(1)
}

for (const f of installers) {
  copyFileSync(join(srcDir, f.name), join(outDir, f.name))
  console.log(`[copy-installer] ${outDir}/${f.name}`)
}
console.log(`[copy-installer] 完成（共 ${installers.length} 个）`)
