import { message } from 'antd';

// 错误类型定义
export enum ErrorType {
  NETWORK = 'network',
  VALIDATION = 'validation',
  AUTHENTICATION = 'authentication',
  SERVER = 'server',
  UNKNOWN = 'unknown'
}

// 错误处理配置接口
interface ErrorHandlerConfig {
  type: ErrorType;
  message: string;
  showNotification?: boolean;
  logError?: boolean;
}

// 错误处理工具类
export class ErrorHandler {
  // 处理错误
  static handleError(error: unknown, config?: Partial<ErrorHandlerConfig>): string {
    const defaultConfig: ErrorHandlerConfig = {
      type: ErrorType.UNKNOWN,
      message: '操作失败，请稍后重试',
      showNotification: true,
      logError: true
    };

    const finalConfig = { ...defaultConfig, ...config };

    // 记录错误日志
    if (finalConfig.logError) {
      console.error('Error:', error);
    }

    // 生成错误消息
    let errorMessage = finalConfig.message;

    // 根据错误类型和具体错误信息生成更详细的错误消息
    if (typeof error === 'object' && error !== null) {
      if ('message' in error && typeof error.message === 'string') {
        errorMessage = error.message;
      } else if ('response' in error && typeof error.response === 'object' && error.response !== null) {
        const response = error.response as { data?: { message?: string }, status?: number };
        if (response.data?.message) {
          errorMessage = response.data.message;
        } else if (response.status === 401) {
          errorMessage = '认证失败，请重新登录';
        } else if (response.status === 403) {
          errorMessage = '权限不足，无法执行此操作';
        } else if (response.status === 404) {
          errorMessage = '请求的资源不存在';
        } else if (response.status && response.status >= 500) {
          errorMessage = '服务器错误，请稍后重试';
        }
      } else if ('code' in error && typeof error.code === 'string') {
        if (error.code === 'ECONNABORTED') {
          errorMessage = '请求超时，请检查网络连接';
        } else if (error.code === 'ENOTFOUND') {
          errorMessage = '网络连接失败，请检查网络设置';
        }
      }
    }

    // 显示错误通知
    if (finalConfig.showNotification) {
      message.error(errorMessage);
    }

    return errorMessage;
  }

  // 处理网络错误
  static handleNetworkError(error: unknown): string {
    return this.handleError(error, {
      type: ErrorType.NETWORK,
      message: '网络连接失败，请检查网络设置'
    });
  }

  // 处理验证错误
  static handleValidationError(error: unknown): string {
    return this.handleError(error, {
      type: ErrorType.VALIDATION,
      message: '输入数据验证失败'
    });
  }

  // 处理认证错误
  static handleAuthenticationError(error: unknown): string {
    return this.handleError(error, {
      type: ErrorType.AUTHENTICATION,
      message: '认证失败，请重新登录'
    });
  }

  // 处理服务器错误
  static handleServerError(error: unknown): string {
    return this.handleError(error, {
      type: ErrorType.SERVER,
      message: '服务器错误，请稍后重试'
    });
  }
}
