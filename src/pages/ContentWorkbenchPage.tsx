/**
 * 内容工作台（首页）：全部文章的唯一入口。
 * 列表总览 → 查看预览 / 编辑（跳编辑器）/ 删除。
 */
import React, { useEffect, useState } from 'react';
import { Button, Table, Space, message, Input, Tag, Typography, Radio, Drawer, Popconfirm } from 'antd';
import { EditOutlined, DeleteOutlined, PlusOutlined, EyeOutlined, SendOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { marked } from 'marked';
import { useArticleStore, Article } from '../stores/articleStore';
import { Routes } from '../utils/routes';
import dayjs from 'dayjs';
import WechatPublish from '../components/wechat/WechatPublish';
import QualityReportView from '../components/editor/QualityReportView';

const { Title } = Typography;

const ContentWorkbenchPage: React.FC = () => {
  const navigate = useNavigate();
  const { articles, deleteArticle, updateArticle, isLoading, loadArticles } = useArticleStore();
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [preview, setPreview] = useState<Article | null>(null);
  const [publishTarget, setPublishTarget] = useState<Article | null>(null);

  useEffect(() => {
    let active = true
    loadArticles().finally(() => {
      // 首次首页数据加载完成即视为“应用已可交互”，通知启动层放行
      if (active) {
        window.dispatchEvent(new Event('app:content-ready'))
      }
    })
    return () => {
      active = false
    }
  }, [loadArticles]);

  const filteredArticles = articles.filter(article =>
    (statusFilter === 'all' || article.status === statusFilter) &&
    article.title.toLowerCase().includes(searchText.toLowerCase())
  );

  const handleEdit = (article: Article) => {
    useArticleStore.getState().setCurrentArticle(article);
    navigate(Routes.EDITOR);
  };

  const handleNewArticle = () => {
    useArticleStore.getState().setCurrentArticle(null);
    navigate(Routes.EDITOR);
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteArticle(id);
      message.success('删除成功');
    } catch (error) {
      message.error('删除失败');
    }
  };

  const handlePublishSuccess = async (article: Article) => {
    try {
      await updateArticle(article.id, { status: 'published' });
      message.success('文章已标记为已发布');
    } catch (error) {
      console.error('更新文章状态失败:', error);
    }
  };

  const columns = [
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
      width: 520,
      render: (title: string, record: Article) => (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0 }}>
          <span
            title={title}
            onClick={() => handleEdit(record)}
            style={{
              cursor: 'pointer',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              maxWidth: '30em',
              flexShrink: 0,
            }}
          >
            {title}
          </span>
          {record.qualityReport && <Tag style={{ fontSize: 11, marginRight: 0, flexShrink: 0 }}>已体检</Tag>}
        </div>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      render: (status: string) => (
        <Tag color={status === 'published' ? 'green' : 'orange'}>
          {status === 'published' ? '已发布' : '未发布'}
        </Tag>
      ),
    },
    {
      title: '更新时间',
      dataIndex: 'updatedAt',
      key: 'updatedAt',
      width: 170,
      render: (date: string) => dayjs(date).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: '操作',
      key: 'action',
      width: 320,
      fixed: 'right' as const,
      render: (_: unknown, record: Article) => (
        <Space style={{ marginRight: 16 }}>
          <Button type="link" size="small" icon={<EyeOutlined />} onClick={() => setPreview(record)}>
            查看
          </Button>
          <Button type="link" size="small" icon={<EditOutlined />} onClick={() => handleEdit(record)}>
            编辑
          </Button>
          <Button type="link" size="small" icon={<SendOutlined />} onClick={() => setPublishTarget(record)}>
            发布
          </Button>
          <Popconfirm title="确定删除这篇文章？" onConfirm={() => void handleDelete(record.id)} okText="确定" cancelText="取消">
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>删除</Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div className="page-container" style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden', padding: 0 }}>
      <div style={{ flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>内容工作台</Title>
      </div>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', padding: 24 }}>
        <div style={{ flexShrink: 0, marginBottom: 16, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <Input.Search
              placeholder="搜索文章"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              allowClear
              style={{ width: 300 }}
            />
            <Radio.Group value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
              <Radio.Button value="all">全部</Radio.Button>
              <Radio.Button value="draft">未发布</Radio.Button>
              <Radio.Button value="published">已发布</Radio.Button>
            </Radio.Group>
          </div>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleNewArticle}>
            新建文章
          </Button>
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          <Table
            columns={columns}
            dataSource={filteredArticles}
            rowKey="id"
            loading={isLoading}
            scroll={{ x: 'max-content' }}
            pagination={{ pageSize: 15 } as any}
            locale={{ emptyText: searchText ? '没有匹配的文章' : statusFilter === 'published' ? '暂无已发布文章' : statusFilter === 'draft' ? '暂无未发布文章' : '还没有文章——去爆文工坊或热点雷达生成一篇，或点右上角新建' }}
          />
        </div>
      </div>

      <Drawer
        title={preview?.title}
        placement="right"
        width={560}
        open={!!preview}
        onClose={() => setPreview(null)}
        extra={
          preview && (
            <Button type="primary" size="small" icon={<EditOutlined />} onClick={() => handleEdit(preview)}>
              编辑
            </Button>
          )
        }
      >
        {preview?.qualityReport && (
          <div style={{ marginBottom: 16, padding: '8px 12px', background: '#f6ffed', borderRadius: 6 }}>
            <QualityReportView report={preview.qualityReport} />
          </div>
        )}
        <div className="hot-analysis" dangerouslySetInnerHTML={{ __html: marked(preview?.content || '') as string }} />
      </Drawer>

      {publishTarget && (
        <WechatPublish
          key={publishTarget.id}
          visible
          title={publishTarget.title}
          content={publishTarget.content}
          onClose={() => setPublishTarget(null)}
          onSuccess={() => void handlePublishSuccess(publishTarget)}
        />
      )}
    </div>
  );
};

export default ContentWorkbenchPage;
