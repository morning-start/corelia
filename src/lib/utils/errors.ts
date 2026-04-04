/**
 * 统一错误处理
 * 
 * 提供标准化的错误类型和处理机制
 */

/**
 * 应用级错误类型
 */
export enum ErrorType {
  /** 网络错误 */
  NETWORK = 'NETWORK_ERROR',
  /** 权限错误 */
  PERMISSION = 'PERMISSION_ERROR',
  /** 数据错误 */
  DATA = 'DATA_ERROR',
  /** 配置错误 */
  CONFIG = 'CONFIG_ERROR',
  /** 未知错误 */
  UNKNOWN = 'UNKNOWN_ERROR',
}

/**
 * 标准化错误对象
 */
export interface AppError {
  /** 错误类型 */
  type: ErrorType;
  /** 错误消息 */
  message: string;
  /** 原始错误（可选） */
  original?: unknown;
  /** 错误发生时间 */
  timestamp: number;
  /** 额外上下文信息 */
  context?: Record<string, unknown>;
}

/**
 * 创建标准化错误
 */
export function createError(
  type: ErrorType,
  message: string,
  original?: unknown,
  context?: Record<string, unknown>
): AppError {
  return {
    type,
    message,
    original,
    timestamp: Date.now(),
    context,
  };
}

/**
 * 将未知错误转换为 AppError
 */
export function toAppError(error: unknown, fallbackMessage = '操作失败'): AppError {
  if (error && typeof error === 'object' && 'type' in error) {
    return error as AppError;
  }
  
  const message = error instanceof Error ? error.message : String(error);
  return createError(
    ErrorType.UNKNOWN,
    message || fallbackMessage,
    error
  );
}

/**
 * 安全的异步操作包装器
 * 自动捕获错误并返回标准化错误
 */
export async function safeAsync<T>(
  operation: () => Promise<T>,
  errorType: ErrorType = ErrorType.UNKNOWN,
  fallbackMessage = '操作失败'
): Promise<{ data?: T; error?: AppError }> {
  try {
    const data = await operation();
    return { data };
  } catch (error) {
    return {
      error: toAppError(error, fallbackMessage),
    };
  }
}

/**
 * 日志记录错误
 */
export function logError(error: AppError, context = 'Application'): void {
  console.error(`[${context}] ${error.type}: ${error.message}`, {
    timestamp: new Date(error.timestamp).toISOString(),
    original: error.original,
    context: error.context,
  });
}
