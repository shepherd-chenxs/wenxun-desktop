import dayjs from 'dayjs';
import type { ScheduledTask, VirtualBlogger, TopicCategory } from '../../../services/storeService';

/**
 * 兼容旧数据：如果博主数据中只有 topicCategory (名称) 而没有 topicCategoryId (ID)
 */
export const parseBloggers = (bloggers: VirtualBlogger[], topicCategories: TopicCategory[]): VirtualBlogger[] => {
  if (!bloggers || topicCategories.length === 0) return bloggers || [];
  
  return bloggers.map((blogger: VirtualBlogger) => {
    if (!blogger.topic_category_id && blogger.topic_category) {
      const category = topicCategories.find((cat: TopicCategory) => cat.name === blogger.topic_category);
      return {
        ...blogger,
        topic_category_id: category ? category.id : blogger.topic_category,
        topic_category_name: category ? category.name : blogger.topic_category
      };
    }
    return blogger;
  });
};

/**
 * 生成任务表单的基础值
 */
export const generateFormBaseValues = (task: ScheduledTask | null) => {
  if (!task) {
    return {
      name: '',
      topic_source: 'ai',
      topic_category_id: '',
      topic_pack_id: '',
      hot_source: 'weibo',
      schedule_type: 'once',
      word_count: 1000,
      account_id: '',
      blogger_id: '',
      author_name: '',
      cover_media_id: '',
      publish_type: 'draft',
    };
  }

  // 兼容驼峰命名和下划线命名
  const getFieldValue = (camelCase: any, snakeCase: any) => {
    return camelCase !== undefined ? camelCase : snakeCase;
  };

  return {
    name: task.name || '',
    topic_source: getFieldValue((task as any).topicSource, task.topic_source) || 'ai',
    topic_category_id: getFieldValue((task as any).topicCategoryId, task.topic_category_id) || '',
    topic_pack_id: getFieldValue((task as any).topicPackId, task.topic_pack_id) || '',
    hot_source: getFieldValue((task as any).hotSource, task.hot_source) || 'weibo',
    schedule_type: getFieldValue((task as any).scheduleType, task.schedule_type) || 'once',
    word_count: getFieldValue((task as any).wordCount, task.word_count) || 1000,
    account_id: getFieldValue((task as any).accountId, task.account_id) || '',
    blogger_id: getFieldValue((task as any).bloggerId, task.blogger_id) || '',
    author_name: getFieldValue((task as any).authorName, task.author_name) || '',
    cover_media_id: getFieldValue((task as any).coverMediaId, task.cover_media_id) || '',
    publish_type: getFieldValue((task as any).publishType, task.publish_type) || 'draft',
  };
};

/**
 * 生成任务表单的时间值
 */
export const generateFormTimeValues = (task: ScheduledTask | null) => {
  if (!task) {
    return {
      publish_time: dayjs().add(1, 'hour'),
    };
  }

  // 兼容驼峰命名和下划线命名
  const getFieldValue = (camelCase: any, snakeCase: any) => {
    return camelCase !== undefined ? camelCase : snakeCase;
  };
  
  const scheduleType = getFieldValue((task as any).scheduleType, task.schedule_type);
  const publishTime = getFieldValue((task as any).publishTime, task.publish_time);
  const weekDay = getFieldValue((task as any).weekDay, task.week_day);
  const monthDay = getFieldValue((task as any).monthDay, task.month_day);
  
  if (scheduleType === 'once') {
    return {
      publish_time: publishTime ? dayjs(publishTime) : dayjs().add(1, 'hour'),
    };
  } else if (scheduleType === 'daily') {
    return {
      publish_time: publishTime ? dayjs(publishTime) : dayjs().add(1, 'hour'),
    };
  } else if (scheduleType === 'weekly') {
    const taskTime = publishTime ? dayjs(publishTime) : dayjs();
    return {
      week_day: weekDay !== undefined ? weekDay : taskTime.day(),
      weekly_time: taskTime,
    };
  } else if (scheduleType === 'monthly') {
    const taskTime = publishTime ? dayjs(publishTime) : dayjs();
    return {
      month_day: monthDay !== undefined ? monthDay : taskTime.date(),
      monthly_time: taskTime,
    };
  }
  
  return {};
};

/**
 * 生成默认的题材分类
 */
export const getDefaultTopicCategories = () => [
  { id: '1', name: '历史', description: '历史事件、人物传记、历史文化等内容', keywords: ['历史', '古代', '近代', '人物', '文化'], isDefault: true },
  { id: '2', name: '搞钱', description: '赚钱方法、理财投资、副业创业', keywords: ['赚钱', '理财', '投资', '副业', '创业'], isDefault: true },
  { id: '3', name: '情感', description: '情感关系、心理成长、人际交往', keywords: ['情感', '恋爱', '婚姻', '心理', '人际'], isDefault: true },
  { id: '4', name: '认知', description: '思维模式、认知提升、学习方法', keywords: ['认知', '思维', '学习', '成长', '智慧'], isDefault: true },
  { id: '5', name: '工具', description: '效率工具、软件推荐、技术应用', keywords: ['工具', '软件', '效率', '技术', '应用'], isDefault: true },
  { id: '6', name: '世相', description: '社会现象、热点评论、生活观察', keywords: ['社会', '热点', '现象', '评论', '观察'], isDefault: true },
];
