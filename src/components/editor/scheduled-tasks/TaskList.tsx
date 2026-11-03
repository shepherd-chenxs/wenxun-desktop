import React from 'react';
import { Dropdown, Button, Table, Typography, Switch } from 'antd';
import { EditOutlined, DeleteOutlined, PlayCircleOutlined, EllipsisOutlined, FileTextOutlined, ClockCircleOutlined, WechatOutlined, FolderOutlined, FireOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import type { ScheduledTask } from '../../../services/storeService';
import { Routes } from '../../../utils/routes';

const { Text } = Typography;

interface TaskListProps {
  tasks: ScheduledTask[];
  executingId: string | null;
  onEdit: (task: ScheduledTask) => void;
  onDelete: (taskId: string) => void;
  onExecute: (taskId: string) => void;
  onToggleEnable: (task: ScheduledTask) => void;
}

const weekDays = ['日', '一', '二', '三', '四', '五', '六'];

const formatTime = (type: string, time: string) => {
  const d = dayjs(time);
  const hm = d.format('HH:mm');
  switch (type) {
    case 'daily': return `每天 ${hm}`;
    case 'weekly': return `每周${weekDays[d.day()]} ${hm}`;
    case 'monthly': return `每月${d.date()}号 ${hm}`;
    default: return d.format('MM-DD HH:mm');
  }
};

const TaskList: React.FC<TaskListProps> = ({ tasks, executingId, onEdit, onDelete, onExecute, onToggleEnable }) => {
  const navigate = useNavigate();

  const columns = [
    {
      title: '任务名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
      ellipsis: true,
      render: (name: string) => name,
    },
    {
      title: '公众号',
      dataIndex: 'account_name',
      key: 'account_name',
      width: 130,
      render: (name: string) => (
        <Text type="secondary" style={{ fontSize: 13 }}>
          <WechatOutlined style={{ marginRight: 4 }} />{name || '未设置'}
        </Text>
      ),
    },
    {
      title: '题材',
      key: 'topic',
      width: 160,
      render: (_: unknown, task: ScheduledTask) => (
        <Text type="secondary" style={{ fontSize: 13 }}>
          {task.topic_source === 'hot'
            ? <FireOutlined style={{ marginRight: 4 }} />
            : <FolderOutlined style={{ marginRight: 4 }} />}
          {task.topic_category_name || '未分类'}
          {task.topic_source === 'ai' ? ' · AI 生成' : task.topic_source === 'hot' ? ' · 热点雷达' : ' · 话题包'}
        </Text>
      ),
    },
    {
      title: '执行时间',
      key: 'schedule',
      width: 150,
      render: (_: unknown, task: ScheduledTask) => (
        <Text type="secondary" style={{ fontSize: 13 }}>
          <ClockCircleOutlined style={{ marginRight: 4 }} />
          {formatTime(task.schedule_type, task.publish_time)}
        </Text>
      ),
    },
    {
      title: '启用',
      key: 'enabled',
      width: 70,
      render: (_: unknown, task: ScheduledTask) => (
        <Switch
          size="small"
          checked={task.enabled}
          onChange={() => onToggleEnable(task)}
        />
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: 260,
      render: (_: unknown, task: ScheduledTask) => (
        <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          {executingId === task.id ? (
            <Text type="secondary" style={{ fontSize: 12 }}>执行中...</Text>
          ) : (
            <Button type="link" size="small" icon={<PlayCircleOutlined />} onClick={() => onExecute(task.id)}>
              立即执行
            </Button>
          )}
          <Button type="link" size="small" icon={<FileTextOutlined />} onClick={() => navigate(`${Routes.TASK_LOGS}?taskId=${task.id}&taskName=${encodeURIComponent(task.name)}`)}>
            日志
          </Button>
          <Dropdown
            menu={{
              items: [
                { key: 'edit', label: '编辑', icon: <EditOutlined />, onClick: () => onEdit(task) },
                { key: 'delete', label: '删除', icon: <DeleteOutlined />, danger: true, onClick: () => onDelete(task.id) },
              ],
            }}
            placement="bottomRight"
          >
            <Button type="text" size="small" icon={<EllipsisOutlined />} />
          </Dropdown>
        </div>
      ),
    },
  ];

  return (
    <Table
      columns={columns}
      dataSource={tasks}
      rowKey="id"
      size="middle"
      pagination={false}
      locale={{ emptyText: '暂无定时任务' }}
    />
  );
};

export default React.memo(TaskList);
