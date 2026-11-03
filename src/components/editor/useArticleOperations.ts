import { useCallback, useState, useEffect } from 'react';
import { useArticleStore, Article } from '../../stores/articleStore';
import { validateTitle, validateContent } from '../../utils/validation';
import { getAllTemplates } from '../../utils/typesetting';
import { ErrorHandler, ErrorType } from '../../utils/errorHandler';
import { message } from 'antd';

interface UseArticleOperationsReturn {
  title: string;
  content: string;
  showTypesettingModal: boolean;
  selectedTypesettingTemplate: string;
  templates: any[];
  appliedTemplate: string;
  setTitle: (title: string) => void;
  setContent: (content: string) => void;
  setShowTypesettingModal: (show: boolean) => void;
  setSelectedTypesettingTemplate: (template: string) => void;
  setAppliedTemplate: (template: string) => void;
  handleSelectArticle: (article: Article) => void;
  handleNewArticle: () => void;
  handleSave: () => Promise<void>;
  handleAIGenerate: (aiTitle: string, aiContent: string) => void;
  handleSmartTypesetting: () => void;
  applyTypesettingTemplate: () => void;
}

export const useArticleOperations = (): UseArticleOperationsReturn => {
  const {
    currentArticle,
    setCurrentArticle,
    addArticle,
    updateArticle,
    loadArticles,
  } = useArticleStore();
  
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [showTypesettingModal, setShowTypesettingModal] = useState(false);
  const [selectedTypesettingTemplate, setSelectedTypesettingTemplate] = useState('default');
  const templates = getAllTemplates();
  const [appliedTemplate, setAppliedTemplate] = useState('default');

  useEffect(() => {
    loadArticles();
  }, [loadArticles]);

  useEffect(() => {
    if (currentArticle) {
      setTitle(currentArticle.title);
      setContent(currentArticle.content);
    } else {
      setTitle('');
      setContent('');
    }
  }, [currentArticle]);

  const handleSelectArticle = useCallback((article: Article) => {
    setCurrentArticle(article);
  }, [setCurrentArticle]);

  const handleNewArticle = useCallback(() => {
    setCurrentArticle(null);
    setTitle('');
    setContent('');
  }, [setCurrentArticle]);

  const handleSave = async () => {
    const titleValidation = validateTitle(title);
    if (!titleValidation.valid) {
      message.error(titleValidation.error);
      return;
    }

    const contentValidation = validateContent(content);
    if (!contentValidation.valid) {
      message.error(contentValidation.error);
      return;
    }

    try {
      if (currentArticle) {
        await updateArticle(currentArticle.id, { title, content });
        message.success('保存成功');
      } else {
        await addArticle({ title, content, status: 'draft' });
        message.success('保存成功');
      }
    } catch (error) {
      ErrorHandler.handleError(error, {
        type: ErrorType.UNKNOWN,
        message: '保存文章失败'
      });
    }
  };

  const handleAIGenerate = (aiTitle: string, aiContent: string) => {
    setTitle(aiTitle);
    setContent(aiContent);
  };

  const handleSmartTypesetting = () => {
    const contentValidation = validateContent(content);
    if (!contentValidation.valid) {
      return;
    }
    setShowTypesettingModal(true);
  };

  const applyTypesettingTemplate = () => {
    const contentValidation = validateContent(content);
    if (!contentValidation.valid) {
      return;
    }

    // 只更新模板选择，样式会通过CSS自动应用到预览区
    setAppliedTemplate(selectedTypesettingTemplate);
    setShowTypesettingModal(false);
  };

  return {
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
    setAppliedTemplate,
    handleSelectArticle,
    handleNewArticle,
    handleSave,
    handleAIGenerate,
    handleSmartTypesetting,
    applyTypesettingTemplate,
  };
};
