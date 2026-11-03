export interface Article {
  id: string
  title: string
  content: string
  cover_media_id?: string
  author_name?: string
  category?: string
  tags?: string
  status: string
  created_at: string
  updated_at: string
}

export interface ScheduledTask {
  id: string
  name: string
  schedule_type: string
  publish_time: string
  topic_source: string
  topic_category_id: string
  topic_pack_id?: string
  hot_source?: string
  blogger_id: string
  word_count: number
  account_id: string
  author_name?: string
  cover_media_id: string
  publish_type?: string
  enabled: boolean
  status: string
  week_day?: number
  month_day?: number
  blogger_name: string
  blogger_category_id: string
  account_name: string
  topic_category_name: string
  topic_pack_name?: string
}

export interface TaskExecutionLog {
  id: string
  task_id: string
  task_name: string
  execution_time: string
  status: string
  topic?: string
  article_title?: string
  duration?: number
  result?: string
  error_message?: string
}

export interface TopicCategory {
  id: string
  name: string
  description?: string
  created_at: string
}

export interface TopicPack {
  id: string
  name: string
  category_id: string
  category_name: string
  description?: string
  topics: string
  created_at: string
}

export interface Topic {
  id: string
  pack_id: string
  content: string
  status: string
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

export interface WechatPublishHistoryItem {
  id: string
  article_id: string
  article_title: string
  account_id: string
  publish_time: string
  status: string
}

export interface AIApiKey {
  id: string
  api_key: string
  created_at: string
}

export type { AIGenerationHistory } from './ai'
