import { invoke as tauriInvoke } from '@tauri-apps/api/core'

const isTauriEnv = typeof window !== 'undefined' && window.__TAURI__ !== undefined

export interface Article {
  id: string
  title: string
  content: string
  summary: string
  status: 'draft' | 'published' | 'archived'
  folder_id: string | null
  wechat_article_id: string | null
  created_at: string
  updated_at: string
  cover_image?: string
}

export interface Folder {
  id: string
  name: string
  parent_id: string | null
  created_at: string
}

export interface TopicCategory {
  id: string
  name: string
  description: string
  created_at: string
}

export interface VirtualBlogger {
  id: string
  name: string
  persona: string
  topic_category: string
  topic_category_id: string
  topic_category_name: string
  writing_style: string
  language_style: string
  article_structure: string
  layout_format: string
  tone_keywords: string
  content_bottom_line: string
  default_word_count: number
  is_builtin?: boolean
  preset_key?: string
  lessons?: string
  created_at?: string
  updated_at?: string
}

export interface WechatAccount {
  id: string
  name: string
  app_id: string
  app_secret: string
  default_cover_media_id?: string
  status: string
  created_at: string
}

export interface TopicPack {
  id?: string
  name: string
  category_id: string
  category_name: string
  description?: string
  topics: string
  created_at: string
}

export interface HotSource {
  key: string
  name: string
}

export interface Topic {
  id?: string
  pack_id: string
  content: string
  status: string
  created_at: string
}

export interface ScheduledTask {
  id: string
  name: string
  schedule_type: 'once' | 'daily' | 'weekly' | 'monthly'
  publish_time: string
  topic_source: 'ai' | 'hot' | 'pack'
  topic_category_id: string
  topic_pack_id: string | null
  hot_source: string | null
  blogger_id: string
  word_count: number
  account_id: string
  author_name: string | null
  cover_media_id: string
  publish_type: string
  enabled: boolean
  status: string
  week_day: number | null
  month_day: number | null
  blogger_name: string
  blogger_category_id: string
  account_name: string
  topic_category_name: string
  topic_pack_name: string | null
  created_at: string
  updated_at: string
}

export interface TaskExecutionLog {
  id: string
  task_id: string
  task_name: string
  execution_time: string
  status: 'success' | 'failed'
  publish_status?: string
  topic?: string
  article_title?: string
  duration?: number
  result?: string
  error_message?: string
}

export interface AIApiKey {
  id: string
  api_key: string
  created_at: string
}

export interface AIConfig {
  id: string
  base_url: string
  model: string
  temperature: number
  max_tokens: number
}

export interface GenerationHistory {
  id: string
  topic: string
  content: string
  created_at: string
}

class StoreService {
  private getMockData(cmd: string): unknown {
    switch (cmd) {
      case 'get_topic_categories':
        return []
      case 'get_bloggers':
        return []
      case 'get_articles':
        return []
      case 'get_folders':
        return []
      case 'get_tasks':
        return []
      case 'get_accounts':
        return []
      case 'get_topic_packs':
        return []
      case 'get_topics':
        return []
      case 'get_ai_api_key':
        return null
      case 'get_task_logs':
        return []
      case 'get_generation_history':
        return []
      case 'get_publish_history':
        return []
      case 'add_blogger':
      case 'update_blogger':
      case 'delete_blogger':
      case 'save_article':
      case 'delete_article':
      case 'save_topic_category':
      case 'delete_topic_category':
      case 'save_virtual_blogger':
      case 'delete_virtual_blogger':
      case 'save_wechat_account':
      case 'delete_account':
      case 'save_topic_pack':
      case 'delete_topic_pack':
      case 'save_ai_api_key':
      case 'add_task':
      case 'update_task':
      case 'delete_task':
      case 'save_generation_history':
      case 'clear_generation_history':
        return true
      default:
        return null
    }
  }

  async invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
    if (!isTauriEnv) {
      console.log(`Not in Tauri environment, returning mock data for: ${cmd}`)
      return this.getMockData(cmd)
    }
    
    try {
      console.log(`invoke: ${cmd}`, args)
      const result = await tauriInvoke(cmd, args)
      console.log(`invoke success: ${cmd}`, result)
      return result
    } catch (e: any) {
      console.error(`invoke failed: ${cmd}`, args, e?.message || e)
      
      // 抛出异常，让调用方处理错误
      throw new Error(e?.message || e?.toString() || `${cmd} 执行失败`)
    }
  }

  async getArticles(): Promise<Article[]> {
    return await this.invoke('get_articles') as Article[]
  }

  async getArticle(id: string): Promise<Article | null> {
    return await this.invoke('get_article', { id }) as Article | null
  }

  async saveArticle(article: Article): Promise<boolean> {
    try {
      const result = await this.invoke('save_article', { article }) as boolean;
      return result;
    } catch (error: any) {
      console.error('保存文章失败:', error);
      throw new Error(`保存文章失败: ${error.message || error}`);
    }
  }

  async saveArticles(articles: Article[]): Promise<void> {
    try {
      await this.invoke('save_articles_batch', { articles })
    } catch (error: any) {
      console.error('批量保存文章失败:', error);
      throw new Error(`批量保存文章失败: ${error.message || error}`);
    }
  }

  async deleteArticle(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_article', { id }) as boolean;
    } catch (error: any) {
      console.error('删除文章失败:', error);
      throw new Error(`删除文章失败: ${error.message || error}`);
    }
  }

  async getFolders(): Promise<Folder[]> {
    return await this.invoke('get_folders') as Folder[]
  }

  async saveFolder(folder: Folder): Promise<boolean> {
    return await this.invoke('save_folder', { folder }) as boolean
  }

  async deleteFolder(id: string): Promise<boolean> {
    return await this.invoke('delete_folder', { id }) as boolean
  }

  async getTopicCategories(): Promise<TopicCategory[]> {
    return await this.invoke('get_topic_categories') as TopicCategory[]
  }

  async saveTopicCategory(category: TopicCategory): Promise<boolean> {
    try {
      return await this.invoke('save_topic_category', { category }) as boolean;
    } catch (error: any) {
      console.error('保存题材分类失败:', error);
      throw new Error(`保存题材分类失败: ${error.message || error}`);
    }
  }

  async deleteTopicCategory(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_topic_category', { id }) as boolean;
    } catch (error: any) {
      console.error('删除题材分类失败:', error);
      throw new Error(`删除题材分类失败: ${error.message || error}`);
    }
  }

  async getVirtualBloggers(): Promise<VirtualBlogger[]> {
    return await this.invoke('get_bloggers') as VirtualBlogger[]
  }

  async saveVirtualBlogger(blogger: VirtualBlogger): Promise<boolean> {
    try {
      return await this.invoke('save_blogger', { blogger }) as boolean;
    } catch (error: any) {
      console.error('保存虚拟博主失败:', error);
      throw new Error(`保存虚拟博主失败: ${error.message || error}`);
    }
  }

  async deleteVirtualBlogger(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_blogger', { id }) as boolean;
    } catch (error: any) {
      console.error('删除虚拟博主失败:', error);
      throw new Error(`删除虚拟博主失败: ${error.message || error}`);
    }
  }

  async getWechatAccounts(): Promise<WechatAccount[]> {
    return await this.invoke('get_accounts') as WechatAccount[]
  }

  async saveWechatAccount(account: WechatAccount): Promise<boolean> {
    try {
      return await this.invoke('save_wechat_account', { account }) as boolean;
    } catch (error: any) {
      console.error('保存公众号失败:', error);
      throw new Error(`保存公众号失败: ${error.message || error}`);
    }
  }

  async deleteWechatAccount(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_account', { id }) as boolean;
    } catch (error: any) {
      console.error('删除公众号失败:', error);
      throw new Error(`删除公众号失败: ${error.message || error}`);
    }
  }

  async getTopicPacks(): Promise<TopicPack[]> {
    return await this.invoke('get_topic_packs') as TopicPack[]
  }

  async getHotSources(): Promise<HotSource[]> {
    return await this.invoke('get_hot_sources') as HotSource[]
  }

  async getTopics(packId?: string): Promise<Topic[]> {
    console.log('[StoreService] getTopics called with packId:', packId)
    const args = packId ? { packId: packId } : {}
    console.log('[StoreService] Invoking get_topics with args:', args)
    const result = await this.invoke('get_topics', args) as Topic[]
    console.log('[StoreService] getTopics returned', result.length, 'topics')
    return result
  }

  async saveTopicPack(pack: TopicPack): Promise<boolean> {
    try {
      return await this.invoke('save_topic_pack', { pack }) as boolean;
    } catch (error: any) {
      console.error('保存话题包失败:', error);
      throw new Error(`保存话题包失败: ${error.message || error}`);
    }
  }

  async deleteTopicPack(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_topic_pack', { id }) as boolean;
    } catch (error: any) {
      console.error('删除话题包失败:', error);
      throw new Error(`删除话题包失败: ${error.message || error}`);
    }
  }

  async getScheduledTasks(): Promise<ScheduledTask[]> {
    return await this.invoke('get_tasks') as ScheduledTask[]
  }

  async saveScheduledTask(task: ScheduledTask): Promise<boolean> {
    console.log('Saving task:', task);
    try {
      const now = new Date().toISOString();
      if (task.id) {
        const taskWithUpdate = { ...task, updated_at: now };
        const result = await this.invoke('update_task', { task: taskWithUpdate }) as boolean;
        console.log('Update result:', result);
        return result;
      } else {
        const taskWithTimestamps = { ...task, created_at: now, updated_at: now };
        const result = await this.invoke('add_task', { task: taskWithTimestamps }) as boolean;
        console.log('Add result:', result);
        return result;
      }
    } catch (error: any) {
      console.error('保存定时任务失败:', error);
      throw new Error(`保存定时任务失败: ${error.message || error}`);
    }
  }

  async deleteScheduledTask(id: string): Promise<boolean> {
    try {
      return await this.invoke('delete_task', { id }) as boolean;
    } catch (error: any) {
      console.error('删除定时任务失败:', error);
      throw new Error(`删除定时任务失败: ${error.message || error}`);
    }
  }

  async executeTask(id: string): Promise<void> {
    try {
      await this.invoke('execute_task', { id });
    } catch (error: any) {
      console.error('执行任务失败:', error);
      throw new Error(`执行任务失败: ${error.message || error}`);
    }
  }

  async getTaskExecutionLogs(taskId?: string): Promise<TaskExecutionLog[]> {
    return await this.invoke('get_task_logs', { task_id: taskId || null }) as TaskExecutionLog[]
  }

  async clearTaskLogs(taskId?: string): Promise<boolean> {
    try {
      return await this.invoke('clear_task_logs', { task_id: taskId }) as boolean;
    } catch (error: any) {
      console.error('清除任务日志失败:', error);
      throw new Error(`清除任务日志失败: ${error.message || error}`);
    }
  }

  async getAIApiKey(): Promise<AIApiKey | null> {
    return await this.invoke('get_ai_api_key') as AIApiKey | null
  }

  async saveAIApiKey(apiKey: AIApiKey): Promise<boolean> {
    try {
      return await this.invoke('save_ai_api_key', { apiKey }) as boolean;
    } catch (error: any) {
      console.error('保存AI API密钥失败:', error);
      throw new Error(`保存AI API密钥失败: ${error.message || error}`);
    }
  }

  async getAIConfig(): Promise<AIConfig | null> {
    try {
      return await this.invoke('get_ai_config') as AIConfig | null;
    } catch (error: any) {
      console.error('获取AI配置失败:', error);
      return null;
    }
  }

  async saveAIConfig(config: AIConfig): Promise<boolean> {
    try {
      return await this.invoke('save_ai_config', { config }) as boolean;
    } catch (error: any) {
      console.error('保存AI配置失败:', error);
      throw new Error(`保存AI配置失败: ${error.message || error}`);
    }
  }

  async getGenerationHistory(): Promise<GenerationHistory[]> {
    return await this.invoke('get_generation_history') as GenerationHistory[]
  }

  async saveGenerationHistory(history: GenerationHistory): Promise<boolean> {
    try {
      return await this.invoke('save_generation_history', { history }) as boolean;
    } catch (error: any) {
      console.error('保存生成历史失败:', error);
      throw new Error(`保存生成历史失败: ${error.message || error}`);
    }
  }

  async clearGenerationHistory(): Promise<boolean> {
    try {
      return await this.invoke('clear_generation_history') as boolean;
    } catch (error: any) {
      console.error('清空生成历史失败:', error);
      throw new Error(`清空生成历史失败: ${error.message || error}`);
    }
  }

  async generateArticle(topic: string, wordCount: number, style: string): Promise<string> {
    return await this.invoke('generate_article', { topic, wordCount, style }) as string
  }

  async generateTitle(content: string): Promise<string> {
    return await this.invoke('generate_title', { content }) as string
  }

  async generateSummary(content: string): Promise<string> {
    return await this.invoke('generate_summary', { content }) as string
  }

  async wechatUploadMedia(accountId: string, media: { name: string; type: string; data: number[] }): Promise<{ mediaId?: string; error?: string }> {
    // 将 MIME 类型（如 "image/png"）转为微信API要求的类型（如 "image"）
    const mediaType = media.type.startsWith('image/') ? 'image' : media.type
    return await this.invoke('wechat_upload_media', {
      accountId: accountId,
      name: media.name,
      mediaType: mediaType,
      data: media.data
    }) as { mediaId?: string; error?: string }
  }

  async wechatUploadMediaWithCredentials(appId: string, appSecret: string, media: { name: string; type: string; data: number[] }): Promise<{ mediaId?: string; error?: string }> {
    const mediaType = media.type.startsWith('image/') ? 'image' : media.type
    return await this.invoke('wechat_upload_media_with_app', {
      appId,
      appSecret,
      name: media.name,
      mediaType,
      data: media.data,
    }) as { mediaId?: string; error?: string }
  }

  async wechatPublishArticle(accountId: string, article: { title: string; content: string; coverMediaId?: string; publishType?: string }): Promise<{ articleId?: string; error?: string }> {
    return await this.invoke('wechat_publish', {
      accountId,
      title: article.title,
      content: article.content,
      author: (article as any).author || null,
      coverMediaId: article.coverMediaId || null,
      publishType: (article as any).publishType || 'draft',
    }) as { articleId?: string; error?: string }
  }

  async wechatGetArticleList(accountId: string, offset: number, count: number): Promise<{ articles: any[]; error?: string }> {
    return await this.invoke('wechat_get_article_list', { account_id: accountId, offset, count }) as { articles: any[]; error?: string }
  }

  async wechatGetArticleDetail(accountId: string, articleId: string): Promise<{ content?: string; error?: string }> {
    return await this.invoke('wechat_get_article_detail', { account_id: accountId, article_id: articleId }) as { content?: string; error?: string }
  }

  async getPublicIP(): Promise<string> {
    return await this.invoke('get_public_ip') as string
  }

  async getLogPath(): Promise<string> {
    return await this.invoke('get_log_path') as string
  }

  async openLogDir(): Promise<void> {
    await this.invoke('open_log_dir')
  }

  async verifyWechatAccount(appId: string, appSecret: string): Promise<string> {
    return await this.invoke('verify_wechat_account', { appId, appSecret }) as string
  }

}

export const storeService = new StoreService()
