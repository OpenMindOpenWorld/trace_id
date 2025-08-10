//! 追踪ID上下文管理
//!
//! 使用 `tokio::task_local` 提供与Web框架无关的追踪ID上下文管理。

use crate::trace_id::TraceId;
use tokio::task_local;

// 使用tokio的task_local来存储当前请求的trace_id
task_local! {
    static CURRENT_TRACE_ID: TraceId;
}

/// 获取当前追踪ID
///
/// 从当前异步任务的上下文中获取trace_id。
/// 如果当前不在追踪上下文中，则记录一个警告并生成一个新的trace_id。
///
/// # 返回
/// 当前请求的追踪ID
pub fn get_trace_id() -> TraceId {
    CURRENT_TRACE_ID
        .try_with(|trace_id| trace_id.clone())
        .unwrap_or_else(|_| {
            // 如果不在追踪上下文中，记录警告并生成新的trace_id
            tracing::warn!("TraceId not found in task-local context. Generating a new one. This might indicate a logic error where a function is called outside of a traced request scope.");
            TraceId::new()
        })
}

/// 在指定的追踪上下文中执行异步操作
///
/// # 参数
/// * `trace_id` - 要设置的追踪ID
/// * `future` - 要执行的异步操作
///
/// # 返回
/// 异步操作的结果
pub async fn with_trace_id<F, T>(trace_id: TraceId, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    CURRENT_TRACE_ID.scope(trace_id, future).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 改进测试：验证在没有上下文时，get_trace_id能回退到生成一个新的、有效的ID
    #[tokio::test]
    async fn test_get_trace_id_outside_context() {
        // 在没有追踪上下文的情况下调用
        let trace_id1 = get_trace_id();

        // 验证返回的是一个有效的TraceId
        assert_eq!(trace_id1.as_str().len(), 32, "ID长度应为32");
        assert!(
            TraceId::from_string_validated(trace_id1.as_str()).is_some(),
            "生成的ID应为有效格式"
        );

        // 再次调用，应该生成一个不同的新ID
        let trace_id2 = get_trace_id();
        assert_ne!(trace_id1, trace_id2, "连续调用应生成不同的ID");
    }

    /// 改进测试：验证with_trace_id在整个异步作用域内（包括await点之后）都保持上下文
    #[tokio::test]
    async fn test_with_trace_id_context_persistence() {
        let expected_trace_id = TraceId::new(); // 使用有效的ID

        let result = with_trace_id(expected_trace_id.clone(), async {
            // 在await之前检查
            let current1 = get_trace_id();
            assert_eq!(current1, expected_trace_id, "ID在await之前应匹配");

            // 模拟异步操作
            tokio::time::sleep(Duration::from_millis(1)).await;

            // 在await之后再次检查
            let current2 = get_trace_id();
            assert_eq!(current2, expected_trace_id, "ID在await之后应保持不变");

            "test_result"
        })
        .await;

        assert_eq!(result, "test_result");

        // 验证在作用域之外，上下文已消失
        let outside_id = get_trace_id();
        assert_ne!(outside_id, expected_trace_id, "上下文不应泄漏到作用域之外");
    }

    /// 新增测试：验证嵌套上下文的正确覆盖和恢复
    #[tokio::test]
    async fn test_nested_trace_id_context() {
        let outer_id = TraceId::new();
        let inner_id = TraceId::new();

        with_trace_id(outer_id.clone(), async {
            // 验证外层上下文
            assert_eq!(get_trace_id(), outer_id, "应处于外层上下文");

            // 进入内层上下文
            with_trace_id(inner_id.clone(), async {
                // 验证内层上下文覆盖了外层
                assert_eq!(get_trace_id(), inner_id, "应处于内层上下文");
            })
            .await;

            // 验证退出内层后，恢复到外层上下文
            assert_eq!(get_trace_id(), outer_id, "应恢复到外层上下文");
        })
        .await;
    }

    /// 新增测试：验证并发任务之间的上下文隔离
    #[tokio::test]
    async fn test_concurrent_trace_id_isolation() {
        let mut handles = vec![];
        const NUM_TASKS: usize = 50;

        for _ in 0..NUM_TASKS {
            let trace_id = TraceId::new();
            let trace_id_clone = trace_id.clone();

            let handle = tokio::spawn(async move {
                with_trace_id(trace_id_clone, async move {
                    // 随机等待一段时间，增加任务交错执行的可能性
                    tokio::time::sleep(Duration::from_millis(fastrand::u64(1..10))).await;

                    // 验证当前任务的上下文是否正确
                    let current_id = get_trace_id();
                    assert_eq!(current_id, trace_id, "并发任务中的ID应保持隔离和正确");
                })
                .await;
            });
            handles.push(handle);
        }

        // 等待所有并发任务完成
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
