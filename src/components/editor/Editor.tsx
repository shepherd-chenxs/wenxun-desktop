import React, { useState } from 'react';
import { Layout, Button, Modal, Typography, message } from 'antd';
import { SaveOutlined, UnorderedListOutlined } from '@ant-design/icons';
import EditorHeader from './EditorHeader';
import EditorContent from './EditorContent';
import AIGenerator from '../ai/AIGenerator';
import SmartTypesettingModal from '../editor/SmartTypesettingModal';
import QualityReportView from './QualityReportView';
import { useArticleOperations } from './useArticleOperations';
import { useArticleStore } from '../../stores/articleStore';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

const { Content } = Layout;
const { Text, Paragraph } = Typography;

const Editor: React.FC = () => {
  const {
    title,
    content,
    showTypesettingModal,
    selectedTypesettingTemplate,
    templates,
    appliedTemplate,
    setTitle,
    setContent,
    setShowTypesettingModal,
    setSelectedTypesettingTemplate,
    handleSave,
    handleAIGenerate,
    handleSmartTypesetting,
    applyTypesettingTemplate,
  } = useArticleOperations();

  const currentArticle = useArticleStore((s) => s.currentArticle);
  const [reviseOpen, setReviseOpen] = useState(false);
  const [revising, setRevising] = useState(false);
  const [revisedContent, setRevisedContent] = useState<string | null>(null);

  const handleRevise = async () => {
    if (!currentArticle?.qualityReport || !content.trim()) return;
    setRevising(true);
    try {
      const revised = await tauriInvoke<string>('revise_with_report', {
        content,
        report: currentArticle.qualityReport,
      });
      setRevisedContent(revised.trim());
    } catch (e) {
      message.error((e as Error).message);
    } finally {
      setRevising(false);
    }
  };

  const acceptRevision = () => {
    if (revisedContent) setContent(revisedContent);
    setReviseOpen(false);
    setRevisedContent(null);
    message.success('已应用修订稿，自动保存后可在工作台查看');
  };

  const toolbar = (
    <>
      <AIGenerator onGenerate={handleAIGenerate} />
      <Button icon={<SaveOutlined />} disabled={!title.trim()} onClick={() => void handleSave()}>
        保存
      </Button>
      <Button icon={<UnorderedListOutlined />} disabled={!content.trim()} onClick={handleSmartTypesetting}>
        智能排版
      </Button>
    </>
  );

  return (
    <Layout className="app-layout">
      <Content style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
        <EditorHeader content={content} onRevise={() => setReviseOpen(true)} />
        <div style={{ flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column' }}>
          <EditorContent
            title={title}
            content={content}
            appliedTemplate={appliedTemplate}
            onTitleChange={setTitle}
            onContentChange={setContent}
            toolbar={toolbar}
          />
        </div>
      </Content>
      <SmartTypesettingModal
        visible={showTypesettingModal}
        templates={templates}
        selectedTemplate={selectedTypesettingTemplate}
        originalContent={content}
        onTemplateChange={setSelectedTypesettingTemplate}
        onApply={applyTypesettingTemplate}
        onCancel={() => setShowTypesettingModal(false)}
      />
      <Modal
        title="按体检报告修订"
        open={reviseOpen}
        footer={null}
        onCancel={() => { setReviseOpen(false); setRevisedContent(null) }}
        width={1100}
      >
        {revising ? (
          <div style={{ textAlign: 'center', padding: 40 }}>AI 正在按报告修订，通常需要 20-60 秒…</div>
        ) : revisedContent ? (
          <div>
            <div style={{ display: 'flex', gap: 12, marginBottom: 12 }}>
              <Button onClick={() => setRevisedContent(null)}>重新修订</Button>
              <Button type="primary" onClick={acceptRevision}>接受修订（覆盖当前内容）</Button>
              <Text type="secondary" style={{ fontSize: 12, alignSelf: 'center' }}>接受后自动保存；不满意可放弃，原稿不受影响</Text>
            </div>
            <div style={{ display: 'flex', gap: 12, maxHeight: '60vh' }}>
              <div style={{ flex: 1, overflow: 'auto', border: '1px solid #eee', borderRadius: 6, padding: 12 }}>
                <Text type="secondary">原稿</Text>
                <div className="hot-analysis" dangerouslySetInnerHTML={{ __html: marked(content) as string }} />
              </div>
              <div style={{ flex: 1, overflow: 'auto', border: '1px solid #b7eb8f', borderRadius: 6, padding: 12, background: '#fcfff5' }}>
                <Text type="secondary">修订稿</Text>
                <div className="hot-analysis" dangerouslySetInnerHTML={{ __html: marked(revisedContent) as string }} />
              </div>
            </div>
          </div>
        ) : (
          <div>
            <Paragraph type="secondary">AI 将只针对报告点名的问题做局部修订，保留原稿结构与已达标部分。修订稿会先给你对比，确认后才会覆盖。</Paragraph>
            <div style={{ maxHeight: '45vh', overflow: 'auto', background: '#fafafa', borderRadius: 6, padding: 12 }}>
              <QualityReportView report={currentArticle?.qualityReport || ''} />
            </div>
            <Button type="primary" style={{ marginTop: 12 }} onClick={() => void handleRevise()}>开始修订</Button>
          </div>
        )}
      </Modal>
    </Layout>

  );
};

export default Editor;
