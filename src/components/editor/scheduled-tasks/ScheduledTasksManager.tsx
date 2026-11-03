import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Layout, Button, Drawer, message, Typography } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { storeService } from '../../../services/storeService';
import type { ScheduledTask, WechatAccount, VirtualBlogger, TopicCategory, HotSource } from '../../../services/storeService';
import TaskList from './TaskList';
import TaskForm from './TaskForm';

const { Content } = Layout;
const { Title } = Typography;

const ScheduledTasksManager: React.FC = () => {
  const [drawerVisible, setDrawerVisible] = useState(false);
  const [editingTask, setEditingTask] = useState<ScheduledTask | null>(null);
  const [topicSource, setTopicSource] = useState<'ai' | 'hot'>('ai');
  const [coverMediaId, setCoverMediaId] = useState('');
  const [uploadingCover, setUploadingCover] = useState(false);
  const [executingId, setExecutingId] = useState<string | null>(null);
  const coverFileInputRef = useRef<HTMLInputElement>(null);

  // 本地状态
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [accounts, setAccounts] = useState<WechatAccount[]>([]);
  const [bloggers, setBloggers] = useState<VirtualBlogger[]>([]);
  const [hotSources, setHotSources] = useState<HotSource[]>([]);
  const [topicCategories, setTopicCategories] = useState<TopicCategory[]>([]);

  // 初始加载数据
  const loadAllData = useCallback(async () => {
    try {
      const [taskList, accountList, categoryList, sourceList] = await Promise.all([
        storeService.getScheduledTasks(),
        storeService.getWechatAccounts(),
        storeService.getTopicCategories(),
        storeService.getHotSources(),
      ]);
      
      setTasks(taskList);
      setAccounts(accountList);
      setTopicCategories(categoryList);
      setHotSources(sourceList);

      // 加载博主
      const bloggerList = await storeService.getVirtualBloggers();
      setBloggers(bloggerList);
    } catch (error) {
      console.error('加载数据失败:', error);
      message.error('加载数据失败');
    }
  }, []);

  useEffect(() => {
    loadAllData();
  }, [loadAllData]);

  // 处理添加任务
  const handleAdd = () => {
    setEditingTask(null);
    setTopicSource('ai');
    setCoverMediaId('');
    setDrawerVisible(true);
  };

  // 处理编辑任务
  const handleEdit = (task: ScheduledTask) => {
    console.log('Editing task:', task);
    console.log('Current accounts:', accounts);
    console.log('Current bloggers:', bloggers);
    console.log('Current topicCategories:', topicCategories);
    setEditingTask(task);
    setCoverMediaId(task.cover_media_id || '');
    setTopicSource(task.topic_source === 'hot' ? 'hot' : 'ai');
    setDrawerVisible(true);
  };

  // 处理保存任务
  const handleSave = async (taskData: any) => {
    try {
      if (editingTask) {
        // 更新任务
        await storeService.saveScheduledTask({ ...taskData, id: editingTask.id, created_at: (editingTask as any).created_at });
        message.success('任务更新成功');
      } else {
        // 添加任务
        await storeService.saveScheduledTask(taskData);
        message.success('任务添加成功');
      }

      setDrawerVisible(false);
      await loadAllData();
    } catch (error: any) {
      console.error('保存任务失败:', error);
      message.error(`保存任务失败: ${error?.message || error}`);
    }
  };

  // 处理删除任务
  const handleDelete = async (taskId: string) => {
    try {
      await storeService.deleteScheduledTask(taskId);
      message.success('任务删除成功');
      await loadAllData();
    } catch (error) {
      console.error('删除任务失败:', error);
      message.error('删除任务失败');
    }
  };

  // 处理执行任务
  const handleExecute = async (taskId: string) => {
    setExecutingId(taskId);
    try {
      await storeService.executeTask(taskId);
      message.success('任务执行成功');
      await loadAllData();
    } catch (error) {
      console.error('执行任务失败:', error);
      message.error('执行任务失败');
    } finally {
      setExecutingId(null);
    }
  };

  // 处理启用/禁用任务
  const handleToggleEnable = async (task: ScheduledTask) => {
    try {
      await storeService.saveScheduledTask({ ...task, enabled: !task.enabled });
      message.success(`任务已${!task.enabled ? '启用' : '禁用'}`);
      await loadAllData();
    } catch (error) {
      console.error('切换任务状态失败:', error);
      message.error('切换任务状态失败');
    }
  };

  // 处理上传封面
  const handleUploadCover = (accountId: string | undefined) => {
    if (!accountId) {
      message.error('请先在发布设置中选择公众号');
      return;
    }
    
    if (coverFileInputRef.current) {
      coverFileInputRef.current.dataset.accountId = accountId;
      coverFileInputRef.current.click();
    }
  };

  // 处理封面文件选择
  const handleCoverFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const selectedAccountId = e.target.dataset?.accountId;
    
    if (!selectedAccountId) {
      message.error('请先在发布设置中选择公众号');
      setUploadingCover(false);
      if (coverFileInputRef.current) {
        coverFileInputRef.current.value = '';
      }
      return;
    }

    setUploadingCover(true);
    try {
      const arrayBuffer = await file.arrayBuffer();
      
      const result = await storeService.wechatUploadMedia(selectedAccountId, {
        name: file.name,
        type: file.type,
        data: Array.from(new Uint8Array(arrayBuffer))
      });

      if (result && result.mediaId) {
        setCoverMediaId(result.mediaId);
        message.success('封面上传成功');
      } else {
        message.error(`上传失败: ${result?.error || '未知错误'}`);
      }
    } catch (error: any) {
      console.error('上传封面失败:', error);
      message.error(`上传失败: ${error.message}`);
    } finally {
      setUploadingCover(false);
      if (coverFileInputRef.current) {
        coverFileInputRef.current.value = '';
      }
    }
  };

  return (
    <Content style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* 固定标题栏 */}
      <div style={{ flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px 24px', background: '#fff', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={4} style={{ margin: 0 }}>定时任务</Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>新建任务</Button>
      </div>
      {/* 可滚动内容 */}
      <div style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        <TaskList
          tasks={tasks}
          executingId={executingId}
          onEdit={handleEdit}
          onDelete={handleDelete}
          onExecute={handleExecute}
          onToggleEnable={handleToggleEnable}
        />

      <Drawer
        title={editingTask ? '编辑定时任务' : '新建定时任务'}
        open={drawerVisible}
        onClose={() => setDrawerVisible(false)}
        width={600}
        key={editingTask?.id || 'new'}
      >
        <TaskForm
          task={editingTask}
          accounts={accounts}
          bloggers={bloggers}
          hotSources={hotSources}
          topicCategories={topicCategories}
          onSave={handleSave}
          onCancel={() => setDrawerVisible(false)}
          initialTopicSource={topicSource}
          initialCoverMediaId={coverMediaId}
          onTopicSourceChange={setTopicSource}
          uploadingCover={uploadingCover}
          onUploadCover={handleUploadCover}
        />
      </Drawer>

        <input
          ref={coverFileInputRef}
          type="file"
          accept="image/*"
          style={{ display: 'none' }}
          onChange={handleCoverFileChange}
        />
      </div>
    </Content>
  );
};

export default React.memo(ScheduledTasksManager);
