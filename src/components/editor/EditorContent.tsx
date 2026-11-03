import React from 'react';
import { Input, Space } from 'antd';
import MarkdownEditor from '../MarkdownEditor';
import './templateStyles.css';

interface EditorContentProps {
  title: string;
  content: string;
  appliedTemplate: string;
  onTitleChange: (title: string) => void;
  onContentChange: (content: string) => void;
  toolbar?: React.ReactNode;
}

const EditorContent: React.FC<EditorContentProps> = ({
  title,
  content,
  appliedTemplate,
  onTitleChange,
  onContentChange,
  toolbar,
}) => {
  return (
    <div className="editor-area">
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
        <Input
          placeholder="文章标题（50 字以内）"
          value={title}
          onChange={(e) => onTitleChange(e.target.value)}
          className="title-input"
          maxLength={50}
          style={{ width: 600, flex: '0 1 600px', maxWidth: '100%' }}
        />
        {toolbar && <Space style={{ flexShrink: 0, marginLeft: 'auto' }}>{toolbar}</Space>}
      </div>
      <div className="editor-wrapper">
        <div className="editor-with-template">
          <MarkdownEditor
            content={content}
            onChange={onContentChange}
            templateClass={`editor-template-${appliedTemplate}`}
          />
        </div>
        <div style={{ padding: '8px 16px', fontSize: 12, color: '#666', borderTop: '1px solid #f0f0f0', textAlign: 'right' }}>
          字数: {content.length}
        </div>
      </div>
    </div>
  );
};

export default EditorContent;
