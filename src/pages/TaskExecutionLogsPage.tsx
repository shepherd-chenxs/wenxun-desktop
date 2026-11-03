import React, { useState, useEffect } from 'react';
import { Layout, Button, Table, Space, message, Tag, Typography, Modal } from 'antd';
import { ArrowLeftOutlined, DeleteOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';
import dayjs from 'dayjs';
import type { TaskExecutionLog } from '../services/storeService';
import { storeService } from '../services/storeService';

const { Content } = Layout;
const { Title } = Typography;

const TaskExecutionLogsPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [logs, setLogs] = useState<TaskExecutionLog[]>([]);
  const [taskId, setTaskId] = useState<string | undefined>(undefined);
  const [taskName, setTaskName] = useState<string>('');
  const [detailVisible, setDetailVisible] = useState(false);
  const [detailLog, setDetailLog] = useState<TaskExecutionLog | null>(null);

  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const id = params.get('taskId');
    const name = params.get('taskName');

    if (id) {
      setTaskId(id);
      setTaskName(name || '');
    }

    loadLogs(id || undefined);
  }, [location]);

  const loadLogs = async (filterTaskId?: string) => {
    try {
      const data = await storeService.getTaskExecutionLogs(filterTaskId);
      setLogs(data || []);
    } catch (error) {
      console.error('加载执行日志失败:', error);
      message.error('加载执行日志失败');
    }
  };

  const handleClearLogs = async () => {
    try {
      await storeService.clearTaskLogs(taskId);
      message.success('清空日志成功');
      loadLogs(taskId);
    } catch (error) {
      message.error('清空日志失败');
    }
  };

  const columns = [
    {
      title: '文章标题',
      dataIndex: 'article_title',
      key: 'article_title',
      width: 200,
      ellipsis: true,
      render: (title: string) => title || '-',
    },
    {
      title: '执行时间',
      dataIndex: 'execution_time',
      key: 'execution_time',
      width: 180,
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
    },
    {
      title: '生成状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      render: (status: string) => (
        <Tag color={status === 'success' ? 'success' : 'error'}>
          {status === 'success' ? '成功' : '失败'}
        </Tag>
      ),
    },
    {
      title: '发布状态',
      dataIndex: 'publish_status',
      key: 'publish_status',
      width: 100,
      render: (s: string) => {
        if (!s) return <Tag>-</Tag>
        if (s === 'success') return <Tag color="success">已发布</Tag>
        if (s === 'draft') return <Tag color="success">已发布</Tag>
        return <Tag color="error">发布失败</Tag>
      },
    },
    {
      title: '耗时',
      dataIndex: 'duration',
      key: 'duration',
      width: 100,
      render: (duration: number) => duration ? `${duration}s` : '-',
    },
    {
      title: '操作',
      key: 'action',
      width: 80,
      render: (_: unknown, record: TaskExecutionLog) => (
        <Button
          type="link"
          size="small"
          icon={<InfoCircleOutlined />}
          onClick={() => { setDetailLog(record); setDetailVisible(true); }}
        >
          详情
        </Button>
      ),
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Content className="page-container">
        <div className="page-header">
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate(-1)}>
            返回
          </Button>
          <Title level={2} style={{ margin: 0 }}>
            {taskName ? `${taskName} - 执行日志` : '执行日志'}
          </Title>
          <Space>
            <Button
              danger
              icon={<DeleteOutlined />}
              onClick={handleClearLogs}
              disabled={logs.length === 0}
            >
              清空日志
            </Button>
          </Space>
        </div>

        <Table
          columns={columns}
          dataSource={logs}
          rowKey="id"
          pagination={{ pageSize: 10, showSizeChanger: true }}
          locale={{ emptyText: '暂无执行日志' }}
        />

        <Modal
          title="执行详情"
          open={detailVisible}
          onCancel={() => setDetailVisible(false)}
          footer={<Button onClick={() => setDetailVisible(false)}>关闭</Button>}
          width={560}
        >
          {detailLog && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <div>
                <Typography.Text strong>任务名称：</Typography.Text>
                <span>{detailLog.task_name}</span>
              </div>
              <div>
                <Typography.Text strong>执行时间：</Typography.Text>
                <span>{dayjs(detailLog.execution_time).format('YYYY-MM-DD HH:mm:ss')}</span>
              </div>
              <div>
                <Typography.Text strong>生成状态：</Typography.Text>
                <Tag color={detailLog.status === 'success' ? 'success' : 'error'}>
                  {detailLog.status === 'success' ? '成功' : '失败'}
                </Tag>
              </div>
              <div>
                <Typography.Text strong>发布状态：</Typography.Text>
                {detailLog.publish_status === 'success' || detailLog.publish_status === 'draft' ? (
                  <Tag color="success">已发布</Tag>
                ) : detailLog.publish_status ? (
                  <Tag color="error">发布失败</Tag>
                ) : (
                  <Tag>-</Tag>
                )}
              </div>
              <div>
                <Typography.Text strong>话题：</Typography.Text>
                <span>{detailLog.topic || '-'}</span>
              </div>
              <div>
                <Typography.Text strong>文章标题：</Typography.Text>
                <span>{detailLog.article_title || '-'}</span>
              </div>
              <div>
                <Typography.Text strong>耗时：</Typography.Text>
                <span>{detailLog.duration ? `${detailLog.duration}s` : '-'}</span>
              </div>
              <div>
                <Typography.Text strong>执行结果：</Typography.Text>
                <pre style={{ whiteSpace: 'pre-wrap', background: '#f5f5f5', padding: 8, borderRadius: 4, margin: '4px 0', maxHeight: 200, overflow: 'auto' }}>
                  {detailLog.result || '-'}
                </pre>
              </div>
              {detailLog.error_message && (
                <div>
                  <Typography.Text strong style={{ color: '#ff4d4f' }}>错误信息：</Typography.Text>
                  <pre style={{ whiteSpace: 'pre-wrap', background: '#fff2f0', padding: 8, borderRadius: 4, margin: '4px 0', maxHeight: 200, overflow: 'auto', color: '#ff4d4f' }}>
                    {detailLog.error_message}
                  </pre>
                </div>
              )}
            </div>
          )}
        </Modal>
      </Content>
    </Layout>
  );
};

export default TaskExecutionLogsPage;