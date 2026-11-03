import React, { useEffect, useMemo } from 'react';
import { Alert, Form, Input, Select, DatePicker, TimePicker, Radio, InputNumber, Button, Space, message, Tooltip } from 'antd';
import { UploadOutlined, QuestionCircleOutlined } from '@ant-design/icons';
import { Formik, Form as FormikForm, Field } from 'formik';
import * as Yup from 'yup';
import dayjs from 'dayjs';
import type { ScheduledTask, WechatAccount, VirtualBlogger, TopicCategory, HotSource } from '../../../services/storeService';
import { generateFormBaseValues, generateFormTimeValues } from './utils';

const { Option } = Select;
// 表单验证模式
const TaskFormSchema = Yup.object({
  name: Yup.string().required('请输入任务名称'),
  schedule_type: Yup.string().required('请选择调度类型'),
  publish_time: Yup.mixed().required('请选择执行时间'),
  topic_source: Yup.string().required('请选择话题来源'),
  hot_source: Yup.string().when('topic_source', {
    is: 'hot',
    then: (schema) => schema.required('请选择热榜来源'),
  }),
  blogger_id: Yup.string().required('请选择虚拟博主'),
  word_count: Yup.number()
    .required('请输入文章字数')
    .min(500, '文章字数最少500字')
    .max(5000, '文章字数最多5000字'),
  account_id: Yup.string().required('请选择公众号'),
  cover_media_id: Yup.string().required('请上传封面图片或输入素材ID'),
  week_day: Yup.number().when('schedule_type', {
    is: 'weekly',
    then: (schema) => schema.required('请选择星期几'),
  }),
  weekly_time: Yup.mixed().when('schedule_type', {
    is: 'weekly',
    then: (schema) => schema.required('请选择执行时间'),
  }),
  month_day: Yup.number().when('schedule_type', {
    is: 'monthly',
    then: (schema) => schema.required('请选择每月几号'),
  }),
  monthly_time: Yup.mixed().when('schedule_type', {
    is: 'monthly',
    then: (schema) => schema.required('请选择执行时间'),
  }),
});

interface TaskFormProps {
  task: ScheduledTask | null;
  accounts: WechatAccount[];
  bloggers: VirtualBlogger[];
  hotSources: HotSource[];
  topicCategories?: TopicCategory[];
  onSave: (task: any) => Promise<void>;
  onCancel: () => void;
  initialTopicSource: 'ai' | 'hot';
  initialCoverMediaId: string;
  onTopicSourceChange: (source: 'ai' | 'hot') => void;
  uploadingCover: boolean;
  onUploadCover: (accountId: string | undefined) => void;
}

const TaskForm: React.FC<TaskFormProps> = ({
  task,
  accounts,
  bloggers,
  hotSources,
  onSave,
  onCancel,
  initialTopicSource,
  initialCoverMediaId,
  onTopicSourceChange,
  uploadingCover,
  onUploadCover
}) => {
  // 使用ref保存Formik的setFieldValue，以便在useEffect中使用
  const setFieldValueRef = React.useRef<(field: string, value: any) => void>();

  // 当外部传入的initialCoverMediaId变化时（如上传成功后），更新Formik表单中的值
  useEffect(() => {
    if (initialCoverMediaId && setFieldValueRef.current) {
      setFieldValueRef.current('cover_media_id', initialCoverMediaId);
    }
  }, [initialCoverMediaId]);

  // 记忆化初始值，避免每次渲染时创建新对象导致表单重置
  const initialValues = useMemo(() => {
    if (task) {
      const baseValues = generateFormBaseValues(task);
      const timeValues = generateFormTimeValues(task);
      const accountDefaultCover = accounts.find(a => a.id === task.account_id)?.default_cover_media_id || '';
      return {
        ...baseValues,
        ...timeValues,
        // 话题包已下线：编辑旧任务时按 AI 来源继续，避免出现无选项状态
        topic_source: task.topic_source === 'hot' ? 'hot' : 'ai',
        cover_media_id: baseValues.cover_media_id || accountDefaultCover,
      };
    }

    return {
      name: '',
      schedule_type: 'once',
      publish_time: dayjs(),
      topic_source: initialTopicSource,
      topic_category_id: '',
      topic_pack_id: '',
      hot_source: 'weibo',
      blogger_id: '',
      word_count: 1000,
      account_id: '',
      author_name: '',
      cover_media_id: '',
      publish_type: 'draft',
      week_day: 0,
      weekly_time: dayjs(),
      month_day: 1,
      monthly_time: dayjs(),
    };
  }, [task, initialTopicSource, accounts]);

  // 生成初始值
  const getInitialValues = () => initialValues;

  const handleSubmit = async (values: any) => {
    try {
      console.log('Form values:', values);
      
      const selectedBlogger = bloggers.find(b => b.id === values.blogger_id);
      const selectedAccount = accounts.find(a => a.id === values.account_id);
      
      let publish_time: string;
      let week_day: number | undefined;
      let month_day: number | undefined;

      if (values.schedule_type === 'once') {
        publish_time = dayjs(values.publish_time).toISOString();
      } else if (values.schedule_type === 'daily') {
        const today = dayjs();
        const selectedTime = dayjs(values.publish_time);
        publish_time = today.hour(selectedTime.hour()).minute(selectedTime.minute()).second(selectedTime.second()).toISOString();
      } else if (values.schedule_type === 'weekly') {
        const today = dayjs();
        const selectedTime = dayjs(values.weekly_time);
        const daysUntilTarget = (values.week_day - today.day() + 7) % 7;
        const targetDate = today.add(daysUntilTarget, 'day');
        publish_time = targetDate.hour(selectedTime.hour()).minute(selectedTime.minute()).second(selectedTime.second()).toISOString();
        week_day = values.week_day;
      } else if (values.schedule_type === 'monthly') {
        const today = dayjs();
        const selectedTime = dayjs(values.monthly_time);
        let targetDate = today.date(values.month_day);
        if (targetDate.isBefore(today)) {
          targetDate = targetDate.add(1, 'month');
        }
        publish_time = targetDate.hour(selectedTime.hour()).minute(selectedTime.minute()).second(selectedTime.second()).toISOString();
        month_day = values.month_day;
      } else {
        publish_time = dayjs().add(1, 'hour').toISOString();
      }

      const taskData = {
        ...values,
        publish_time,
        week_day,
        month_day,
        cover_media_id: values.cover_media_id,
        enabled: true,
        status: 'pending',
        blogger_name: selectedBlogger?.name || '',
        blogger_category_id: selectedBlogger?.topic_category_id || '',
        topic_category_id: '',
        account_name: selectedAccount?.name || '',
        topic_category_name: selectedBlogger?.name || '',
        topic_pack_id: null,
      };

      console.log('Task data to save:', taskData);
      await onSave(taskData);
    } catch (error) {
      console.error('表单提交失败:', error);
      message.error('提交失败，请检查控制台');
    }
  };

  return (
    <Formik
      key={task?.id || 'new'}
      initialValues={getInitialValues()}
      validationSchema={TaskFormSchema}
      onSubmit={handleSubmit}
    >
      {({ values, handleChange, setFieldValue, errors, touched, handleSubmit: formikHandleSubmit }) => {
        // 将setFieldValue保存到ref中，以便在外部useEffect中使用
        setFieldValueRef.current = setFieldValue;

        const handleTopicSourceChange = (e: any) => {
          const source = e.target.value as 'ai' | 'hot';
          setFieldValue('topic_source', source);
          onTopicSourceChange(source);
        };

        const handleBloggerChange = (bloggerId: string) => {
          const selectedBlogger = bloggers.find(blogger => blogger.id === bloggerId);
          if (selectedBlogger && selectedBlogger.default_word_count) {
            setFieldValue('word_count', selectedBlogger.default_word_count);
          } else {
            setFieldValue('word_count', 1000);
          }
        };

        const handleAccountChange = (accountId: string) => {
          setFieldValue('account_id', accountId);
          const account = accounts.find(a => a.id === accountId);
          setFieldValue('cover_media_id', account?.default_cover_media_id || '');
        };

        return (
          <FormikForm onSubmit={formikHandleSubmit}>
            {task?.topic_source === 'pack' && (
              <Alert
                type="warning"
                showIcon
                style={{ marginBottom: 20 }}
                message="该任务原使用话题包来源，话题包已下线，保存后将改为按当前选择的话题来源执行。"
              />
            )}
            {/* ====== 基础设置 ====== */}
            <div style={{ marginBottom: 32 }}>
              <div style={{ fontSize: 15, fontWeight: 600, color: '#262626', marginBottom: 20, paddingBottom: 8, borderBottom: '2px solid #1677ff' }}>
                基础设置
              </div>

              <Form.Item
                label="任务名称" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                validateStatus={touched.name && errors.name ? 'error' : ''}
                help={touched.name && errors.name ? String(errors.name) : ''}
              >
                <Field name="name" as={Input} placeholder="请输入任务名称" />
              </Form.Item>

              <Form.Item label="调度类型" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}>
                <Radio.Group name="schedule_type" value={values.schedule_type} onChange={handleChange}>
                  <Radio.Button value="once">单次执行</Radio.Button>
                  <Radio.Button value="daily">每天</Radio.Button>
                  <Radio.Button value="weekly">每周</Radio.Button>
                  <Radio.Button value="monthly">每月</Radio.Button>
                </Radio.Group>
              </Form.Item>

              {values.schedule_type === 'once' && (
                <Form.Item
                  label="执行时间" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                  validateStatus={touched.publish_time && errors.publish_time ? 'error' : ''}
                  help={touched.publish_time && errors.publish_time ? String(errors.publish_time) : ''}
                >
                  <Field name="publish_time" as={DatePicker} showTime format="YYYY-MM-DD HH:mm:ss"
                    style={{ width: '100%' }}
                    disabledDate={(current: any) => current && current < dayjs().subtract(1, 'minute')}
                    onChange={(value: any) => setFieldValue('publish_time', value)} />
                </Form.Item>
              )}

              {values.schedule_type === 'daily' && (
                <Form.Item
                  label="执行时间" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                  validateStatus={touched.publish_time && errors.publish_time ? 'error' : ''}
                  help={touched.publish_time && errors.publish_time ? String(errors.publish_time) : ''}
                >
                  <Field name="publish_time" as={TimePicker} format="HH:mm" style={{ width: '100%' }}
                    onChange={(value: any) => setFieldValue('publish_time', value)} />
                </Form.Item>
              )}

              {values.schedule_type === 'weekly' && (
                <>
                  <Form.Item
                    label="星期几" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                    validateStatus={touched.week_day && errors.week_day ? 'error' : ''}
                    help={touched.week_day && errors.week_day ? String(errors.week_day) : ''}
                  >
                    <Field name="week_day" as={Select} placeholder="请选择星期几"
                      onChange={(value: any) => setFieldValue('week_day', value)}>
                      <Option value={0}>周日</Option><Option value={1}>周一</Option><Option value={2}>周二</Option>
                      <Option value={3}>周三</Option><Option value={4}>周四</Option><Option value={5}>周五</Option><Option value={6}>周六</Option>
                    </Field>
                  </Form.Item>
                  <Form.Item
                    label="执行时间" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                    validateStatus={touched.weekly_time && errors.weekly_time ? 'error' : ''}
                    help={touched.weekly_time && errors.weekly_time ? String(errors.weekly_time) : ''}
                  >
                    <Field name="weekly_time" as={TimePicker} format="HH:mm" style={{ width: '100%' }}
                      onChange={(value: any) => setFieldValue('weekly_time', value)} />
                  </Form.Item>
                </>
              )}

              {values.schedule_type === 'monthly' && (
                <>
                  <Form.Item
                    label="每月几号" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                    validateStatus={touched.month_day && errors.month_day ? 'error' : ''}
                    help={touched.month_day && errors.month_day ? String(errors.month_day) : ''}
                  >
                    <Field name="month_day" as={Select} placeholder="请选择每月几号"
                      onChange={(value: any) => setFieldValue('month_day', value)}>
                      {Array.from({ length: 31 }, (_, i) => i + 1).map(day => (
                        <Option key={day} value={day}>{day}号</Option>
                      ))}
                    </Field>
                  </Form.Item>
                  <Form.Item
                    label="执行时间" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                    validateStatus={touched.monthly_time && errors.monthly_time ? 'error' : ''}
                    help={touched.monthly_time && errors.monthly_time ? String(errors.monthly_time) : ''}
                  >
                    <Field name="monthly_time" as={TimePicker} format="HH:mm" style={{ width: '100%' }}
                      onChange={(value: any) => setFieldValue('monthly_time', value)} />
                  </Form.Item>
                </>
              )}
            </div>

            {/* ====== 内容配置 ====== */}
            <div style={{ marginBottom: 32 }}>
              <div style={{ fontSize: 15, fontWeight: 600, color: '#262626', marginBottom: 20, paddingBottom: 8, borderBottom: '2px solid #1677ff' }}>
                内容配置
              </div>

              <Form.Item label="话题来源" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}>
                <Radio.Group name="topic_source" value={values.topic_source} onChange={handleTopicSourceChange}>
                  <Radio.Button value="ai">AI 按人设赛道生成</Radio.Button>
                  <Radio.Button value="hot">热点雷达</Radio.Button>
                </Radio.Group>
              </Form.Item>

              {values.topic_source === 'hot' && (
                <>
                  <Form.Item
                    label="热榜来源" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                    validateStatus={touched.hot_source && errors.hot_source ? 'error' : ''}
                    help={touched.hot_source && errors.hot_source ? String(errors.hot_source) : ''}
                    extra="任务执行时将从该平台实时热榜中自动选题。"
                  >
                    <Select placeholder="请选择热榜来源" value={values.hot_source}
                      onChange={(value) => setFieldValue('hot_source', value)}>
                      {hotSources?.map(source => (<Option key={source.key} value={source.key}>{source.name}</Option>))}
                    </Select>
                  </Form.Item>
                </>
              )}

              <Form.Item
                label="虚拟博主" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                validateStatus={touched.blogger_id && errors.blogger_id ? 'error' : ''}
                help={touched.blogger_id && errors.blogger_id ? String(errors.blogger_id) : ''}
              >
                <Select
                  placeholder="搜索或选择虚拟博主"
                  value={values.blogger_id}
                  showSearch
                  optionFilterProp="children"
                  onChange={(value) => { setFieldValue('blogger_id', value); handleBloggerChange(value); }}>
                  {bloggers?.filter(b => !values.topic_category_id || b.topic_category_id === values.topic_category_id)
                    .map(b => (<Option key={b.id} value={b.id}>{b.name} ({b.topic_category_name || ''})</Option>))}
                </Select>
              </Form.Item>

              <Form.Item
                label="文章字数" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                validateStatus={touched.word_count && errors.word_count ? 'error' : ''}
                help={touched.word_count && errors.word_count ? String(errors.word_count) : ''}
              >
                <InputNumber name="word_count" value={values.word_count}
                  onChange={(value) => setFieldValue('word_count', value)}
                  placeholder="例如：1000" min={500} max={5000} style={{ width: '100%' }} />
              </Form.Item>
            </div>

            {/* ====== 发布设置 ====== */}
            <div style={{ marginBottom: 32 }}>
              <div style={{ fontSize: 15, fontWeight: 600, color: '#262626', marginBottom: 20, paddingBottom: 8, borderBottom: '2px solid #1677ff' }}>
                发布设置
              </div>

              <Form.Item
                label="公众号" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                validateStatus={touched.account_id && errors.account_id ? 'error' : ''}
                help={touched.account_id && errors.account_id ? String(errors.account_id) : ''}
              >
                <Select placeholder="请选择公众号" value={values.account_id}
                  onChange={handleAccountChange}>
                  {accounts?.map(a => (<Option key={a.id} value={a.id}>{a.name}</Option>))}
                </Select>
              </Form.Item>

              <Form.Item label="发布类型" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}>
                <Radio.Group value={values.publish_type || 'draft'}
                  onChange={(e) => setFieldValue('publish_type', e.target.value)}>
                  <Radio.Button value="draft">保存到草稿箱</Radio.Button>
                  <Radio.Button value="mass">提交群发</Radio.Button>
                </Radio.Group>
                <Tooltip title="提交群发需要公众号已完成微信认证并具备群发接口权限（个人订阅号不支持），否则请选择「保存到草稿箱」">
                  <QuestionCircleOutlined style={{ color: '#999', marginLeft: 8, cursor: 'help' }} />
                </Tooltip>
              </Form.Item>

              <Form.Item label="作者名称" labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}>
                <Field name="author_name" as={Input} placeholder="请输入作者名称（可选）" />
              </Form.Item>

              <Form.Item
                label="封面素材" required labelCol={{ flex: '80px' }} wrapperCol={{ flex: 'auto' }}
                validateStatus={touched.cover_media_id && errors.cover_media_id ? 'error' : ''}
                help={touched.cover_media_id && errors.cover_media_id ? String(errors.cover_media_id) : ''}
                extra={<span style={{ fontSize: 12 }}>上传图片将用于本任务生成文章封面</span>}
              >
                <Space.Compact style={{ width: '100%' }}>
                  <Input name="cover_media_id" value={values.cover_media_id}
                    onChange={(e) => setFieldValue('cover_media_id', e.target.value)}
                    placeholder="上传图片后自动填入素材ID，或手动输入" />
                  <Button icon={<UploadOutlined />}
                    onClick={() => {
                      if (!values.account_id) { message.warning('请先选择公众号'); return; }
                      onUploadCover(values.account_id);
                    }}
                    loading={uploadingCover} disabled={!values.account_id}>
                    {uploadingCover ? '上传中' : '上传图片'}
                  </Button>
                </Space.Compact>
              </Form.Item>
            </div>

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, paddingTop: 8, borderTop: '1px solid #f0f0f0' }}>
              <Button onClick={onCancel}>取消</Button>
              <Button type="primary" htmlType="submit">
                {task ? '更新任务' : '添加任务'}
              </Button>
            </div>
          </FormikForm>
        );
      }}
    </Formik>
  );
};

export default React.memo(TaskForm);
