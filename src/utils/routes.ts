export const Routes = {
  HOME: '/',
  EDITOR: '/editor',
  SCHEDULED_TASKS: '/scheduled-tasks',
  TASK_LOGS: '/task-logs',
  WECHAT: '/wechat',
  TOPIC: '/topic',
  BLOGGER: '/blogger',
  TOPIC_PACKS: '/topic-packs',
  TOPIC_PACK_DETAIL: '/topic-packs/:packId',
  AI_SETTINGS: '/ai-settings',
} as const;

export type RoutePath = typeof Routes[keyof typeof Routes];

export const routeList: { path: RoutePath; label: string }[] = [
  { path: Routes.EDITOR, label: '文章编辑' },
  { path: Routes.SCHEDULED_TASKS, label: '定时任务' },
  { path: Routes.TASK_LOGS, label: '执行日志' },
  { path: Routes.WECHAT, label: '微信公众号' },
  { path: Routes.TOPIC, label: '题材配置' },
  { path: Routes.BLOGGER, label: '虚拟博主' },
  { path: Routes.TOPIC_PACKS, label: '话题包管理' },
  { path: Routes.AI_SETTINGS, label: 'AI设置' },
];

export const isValidRoute = (path: string): path is RoutePath => {
  return Object.values(Routes).includes(path as RoutePath);
};
