import React, { useState } from 'react';
import { Button, Empty, message, Modal, Space, Spin, Typography } from 'antd';
import { ArrowLeftOutlined, FileTextOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { Routes } from '../../utils/routes';
import { useArticleStore } from '../../stores/articleStore';
import QualityReportView from './QualityReportView';

const { Title } = Typography;

interface EditorHeaderProps {
  content: string;
  onRevise?: () => void;
}

const EditorHeader: React.FC<EditorHeaderProps> = ({ content, onRevise }) => {
  const navigate = useNavigate();
  const currentArticle = useArticleStore((s) => s.currentArticle);
  const setCurrentArticle = useArticleStore((s) => s.setCurrentArticle);
  const [reportOpen, setReportOpen] = useState(false);
  const [rechecking, setRechecking] = useState(false);
  const hasReport = !!currentArticle?.qualityReport;


  const handleRecheck = async () => {
    if (!currentArticle) return;
    if (!content.trim()) {
      message.warning('当前文章内容为空，无法体检');
      return;
    }
    setRechecking(true);
    try {
      const [report, score] = await tauriInvoke<[string, number]>('recheck_quality', {
        articleId: currentArticle.id,
        content,
      });
      setCurrentArticle({ ...currentArticle, qualityReport: report });
      setReportOpen(true);
      message.success(`体检完成：${score} 分`);
    } catch (e) {
      message.error((e as Error).message);
    } finally {
      setRechecking(false);
    }
  };

  const openReport = () => setReportOpen(true);

  return (
    <div style={{ flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
      <Space>
        <Button icon={<ArrowLeftOutlined />} onClick={() => navigate(Routes.HOME)}>返回工作台</Button>
        <Title level={4} style={{ margin: 0 }}>文章编辑</Title>
      </Space>
      <Space>
        <Button icon={<ReloadOutlined />} loading={rechecking} disabled={!currentArticle} onClick={() => void handleRecheck()}
          title="对当前稿件重新做质量体检，报告会更新">
          重新体检
        </Button>
        <Button
          icon={<FileTextOutlined />}
          disabled={!hasReport}
          onClick={openReport}
          title={hasReport ? '查看质量体检报告' : '当前文章没有体检报告（仅 AI 生成且经过体检的文章才有）'}
        >
          体检报告
        </Button>
        <Button
          disabled={!hasReport || !content.trim()}
          onClick={() => onRevise?.()}
          title={hasReport ? '按体检报告中的问题对文章做局部修订' : '当前文章没有体检报告'}
        >
          按报告修订
        </Button>
      </Space>
      <Modal
        title={`质量体检报告${currentArticle?.title ? ` · ${currentArticle.title}` : ''}`}
        open={reportOpen}
        footer={null}
        onCancel={() => setReportOpen(false)}
        width={680}
      >
        {hasReport ? (
          <QualityReportView report={currentArticle!.qualityReport!} />
        ) : (
          <Empty description="当前文章没有体检报告" />
        )}
        {rechecking && <Spin style={{ marginTop: 12 }} />}
      </Modal>
    </div>
  );
};

export default EditorHeader;
