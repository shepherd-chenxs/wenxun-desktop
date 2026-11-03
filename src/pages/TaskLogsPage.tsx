import React, { useState, useEffect } from 'react';
import { Layout, Button, Table, Space, message, Input, Tag, DatePicker, Select } from 'antd';
import { ArrowLeftOutlined, DeleteOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';
import dayjs from 'dayjs';
import { Routes } from '../utils/routes';
import { storeService } from '../services/storeService';

const { Content } = Layout;
const { RangePicker } = DatePicker;

interface TaskExecutionLog {
  id: string;
  task_id: string;
  task_name: string;
  execution_time: string;
  status: 'success' | 'failed';
  topic?: string;
  article_title?: string;
  duration?: number;
  result?: string;
  error_message?: string;
}

const TaskLogsPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [logs, setLogs] = useState<TaskExecutionLog[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchText, setSearchText] = useState('');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs, dayjs.Dayjs] | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>('');
  
  // 从URL参数获取taskId
  const taskId = new URLSearchParams(location.search).get('taskId');
  const taskName = new URLSearchParams(location.search).get('taskName');

  useEffect(() => {
    loadLogs();
  }, [taskId]);

  const loadLogs = async () => {
    setLoading(true);
    try {
      const data = await storeService.getTaskExecutionLogs(taskId || undefined);
      setLogs(data || []);
    } catch (error) {
      console.error('加载执行日志失败:', error);
      message.error('加载执行日志失败');
    } finally {
      setLoading(false);
    }
  };

  const handleClearLogs = async () => {
    try {
      await storeService.clearTaskLogs(taskId || undefined);
      setLogs([]);
      message.success('清除日志成功');
    } catch (error) {
      message.error('清除日志失败');
    }
  };

  const filteredLogs = logs.filter(log => {
    const matchesSearch = !searchText || 
      log.task_name.toLowerCase().includes(searchText.toLowerCase()) ||
      (log.topic && log.topic.toLowerCase().includes(searchText.toLowerCase())) ||
      (log.article_title && log.article_title.toLowerCase().includes(searchText.toLowerCase()));
    
    const matchesDate = !dateRange || (
      dayjs(log.execution_time).isAfter(dateRange[0]) &&
      dayjs(log.execution_time).isBefore(dateRange[1].add(1, 'day'))
    );
    
    const matchesStatus = !statusFilter || log.status === statusFilter;
    
    return matchesSearch && matchesDate && matchesStatus;
  });

  const columns = [
    {
      title: '任务名称',
      dataIndex: 'task_name',
      key: 'task_name',
      ellipsis: true,
    },
    {
      title: '执行时间',
      dataIndex: 'execution_time',
      key: 'execution_time',
      width: 180,
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 80,
      render: (status: string) => (
        <Tag color={status === 'success' ? 'green' : 'red'}>
          {status === 'success' ? '成功' : '失败'}
        </Tag>
      ),
    },
    {
      title: '话题',
      dataIndex: 'topic',
      key: 'topic',
      ellipsis: true,
      render: (topic: string) => topic || '-',
    },
    {
      title: '文章标题',
      dataIndex: 'article_title',
      key: 'article_title',
      ellipsis: true,
      render: (title: string) => title || '-',
    },
    {
      title: '耗时',
      dataIndex: 'duration',
      key: 'duration',
      width: 80,
      render: (duration: number) => duration ? `${duration}秒` : '-',
    },
    {
      title: '错误信息',
      dataIndex: 'error_message',
      key: 'error_message',
      ellipsis: true,
      render: (error: string) => error || '-',
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Content className="page-container">
        <div className="page-header">
          <Space>
            <Button icon={<ArrowLeftOutlined />} onClick={() => navigate(Routes.SCHEDULED_TASKS)}>
              返回定时任务
            </Button>
            <h2 style={{ margin: 0 }}>
              {taskName ? `任务执行日志 - ${taskName}` : '所有任务执行日志'}
            </h2>
          </Space>
          <Space>
            <Button icon={<ReloadOutlined />} onClick={loadLogs} loading={loading}>
              刷新
            </Button>
            <Button danger icon={<DeleteOutlined />} onClick={handleClearLogs}>
              清除日志
            </Button>
          </Space>
        </div>

        <div style={{ marginBottom: 16, display: 'flex', gap: 16, flexWrap: 'wrap' }}>
          <Input.Search
            placeholder="搜索任务名称、话题或文章标题"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            allowClear
            style={{ width: 300 }}
          />
          <RangePicker
            placeholder={['开始时间', '结束时间']}
            value={dateRange}
            onChange={(dates) => setDateRange(dates as any)}
            style={{ width: 300 }}
          />
          <Select
            placeholder="筛选状态"
            value={statusFilter}
            onChange={setStatusFilter}
            allowClear
            style={{ width: 150 }}
            options={[
              { value: 'success', label: '成功' },
              { value: 'failed', label: '失败' },
            ]}
          />
        </div>

        <Table
          columns={columns}
          dataSource={filteredLogs}
          rowKey="id"
          loading={loading}
          pagination={{ pageSize: 10 }}
          locale={{ emptyText: '暂无执行日志' }}
          scroll={{ x: 1000 }}
        />
      </Content>
    </Layout>
  );
};

export default TaskLogsPage;
