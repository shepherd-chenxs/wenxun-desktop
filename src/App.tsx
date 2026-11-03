import { HashRouter as BrowserRouter, Routes, Route, useNavigate, useLocation } from 'react-router-dom'
import { useEffect, useState, type ReactNode } from 'react'
import { Layout, Menu, Typography } from 'antd'
import {
  EditOutlined, ClockCircleOutlined, SettingOutlined, UserOutlined, ApiOutlined,
  InfoCircleOutlined, FireOutlined,
} from '@ant-design/icons'
import ContentWorkbenchPage from './pages/ContentWorkbenchPage'
import Editor from './components/editor/Editor'
import PublishConfigPage from './pages/PublishConfigPage'
import HotWorkshopPage from './pages/HotWorkshopPage'
import VirtualBloggerPage from './pages/VirtualBloggerPage'
import ScheduledTasksPage from './pages/ScheduledTasksPage'
import TaskExecutionLogsPage from './pages/TaskExecutionLogsPage'
import TopicPackDetailPage from './pages/TopicPackDetailPage'
import AboutPage from './pages/AboutPage'
import './App.css'

const { Sider, Content } = Layout
const { Text } = Typography

type MenuNode = {
  key: string
  icon?: ReactNode
  label: ReactNode
  children?: MenuNode[]
}

const menuItems: MenuNode[] = [
  { key: '/', icon: <EditOutlined />, label: '内容工作台' },
  { key: '/hot-workshop', icon: <FireOutlined />, label: '爆文工坊' },
  { key: '/scheduled-tasks', icon: <ClockCircleOutlined />, label: '定时任务' },
  {
    key: 'config',
    icon: <SettingOutlined />,
    label: '应用配置',
    children: [
      { key: '/blogger', icon: <UserOutlined />, label: '写作人设' },
      { key: '/publish-config', icon: <ApiOutlined />, label: '发布配置' },
      { key: '/about', icon: <InfoCircleOutlined />, label: '关于' },
    ],
  },
]

/**
 * 根据当前路径解析应高亮的菜单项。
 * 优先匹配菜单自身的路径（支持带子路径的页面），
 * 没有对应菜单项的页面按所属功能兜底（编辑器属于内容工作台、执行日志属于定时任务）。
 */
function resolveSelectedKey(pathname: string): string {
  const matched = menuItems
    .flatMap(item => (item.children?.length ? item.children : [item]))
    .map(item => item.key)
    .find(key => (key === '/' ? pathname === key : pathname === key || pathname.startsWith(`${key}/`)))

  if (matched) return matched
  if (pathname.startsWith('/editor')) return '/'
  if (pathname.startsWith('/task-logs')) return '/scheduled-tasks'
  return '/'
}

function findActiveParentKey(selectedKey: string): string | undefined {
  return menuItems.find(item => item.children?.some(child => child.key === selectedKey))?.key
}

function Navigation() {
  const navigate = useNavigate()
  const location = useLocation()
  const selectedKey = resolveSelectedKey(location.pathname)
  const activeParentKey = findActiveParentKey(selectedKey)
  const [openKeys, setOpenKeys] = useState<string[]>(activeParentKey ? [activeParentKey] : [])

  useEffect(() => {
    if (activeParentKey) {
      setOpenKeys(prev => (prev.includes(activeParentKey) ? prev : [...prev, activeParentKey]))
    }
  }, [activeParentKey])

  return (
    <Sider width={200} theme="dark" style={{ overflow: 'hidden' }}>
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ flexShrink: 0, padding: '16px 16px 12px', borderBottom: '1px solid #333', textAlign: 'center' }}>
          <Text style={{ color: 'rgba(255,255,255,0.55)', fontSize: 13, letterSpacing: 2 }}>
            文迅，让创作更简单
          </Text>
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[selectedKey]}
          openKeys={openKeys}
          onOpenChange={setOpenKeys}
          items={menuItems}
          onClick={({ key }) => {
            if (key.startsWith('/')) {
              navigate(key)
            }
          }}
          style={{ flex: 1, overflow: 'auto', borderRight: 0 }}
        />
      </div>
    </Sider>
  )
}

function AppContent() {
  const navigate = useNavigate()

  useEffect(() => {
    console.log('文迅 2.5.1 (Tauri) 已启动')
  }, [navigate])

  return (
    <Layout style={{ width: '100%', height: '100vh' }}>
      <Navigation />
      <Content style={{ overflow: 'auto', background: '#f0f0f0' }}>
        <Routes>
          <Route path="/" element={<ContentWorkbenchPage />} />
          <Route path="/editor" element={<Editor />} />
          <Route path="/hot-workshop" element={<HotWorkshopPage />} />
          <Route path="/scheduled-tasks" element={<ScheduledTasksPage />} />
          <Route path="/task-logs" element={<TaskExecutionLogsPage />} />
          <Route path="/publish-config" element={<PublishConfigPage />} />
          <Route path="/topic-manage/packs/:packId" element={<TopicPackDetailPage />} />
          <Route path="/blogger" element={<VirtualBloggerPage />} />
          <Route path="/about" element={<AboutPage />} />
        </Routes>
      </Content>
    </Layout>
  )
}

function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  )
}

export default App
