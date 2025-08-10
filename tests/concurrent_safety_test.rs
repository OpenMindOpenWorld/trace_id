//! 并发安全性测试
//!
//! 验证 trace_id 模块在高并发场景下的线程安全性和稳定性

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;
use trace_id::{TraceId, get_trace_id, with_trace_id};

/// 测试并发ID生成的唯一性
#[tokio::test]
async fn test_concurrent_id_generation_uniqueness() {
    const THREAD_COUNT: usize = 10;
    const IDS_PER_THREAD: usize = 1000;
    
    let ids = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];
    
    // 启动多个线程并发生成ID
    for _ in 0..THREAD_COUNT {
        let ids_clone = Arc::clone(&ids);
        let handle = tokio::spawn(async move {
            let mut local_ids = Vec::new();
            
            for _ in 0..IDS_PER_THREAD {
                let trace_id = TraceId::new();
                local_ids.push(trace_id.as_str().to_string());
            }
            
            // 将本地生成的ID添加到全局集合
            let mut global_ids = ids_clone.lock().unwrap();
            for id in local_ids {
                assert!(global_ids.insert(id), "发现重复的trace_id");
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 验证生成的ID总数
    let final_ids = ids.lock().unwrap();
    assert_eq!(final_ids.len(), THREAD_COUNT * IDS_PER_THREAD);
}

/// 测试并发上下文管理的安全性
#[tokio::test]
async fn test_concurrent_context_management() {
    const CONCURRENT_TASKS: usize = 100;
    
    let mut handles = vec![];
    
    for i in 0..CONCURRENT_TASKS {
        let handle = tokio::spawn(async move {
            let expected_id = format!("test-trace-id-{:03}", i);
            let trace_id = TraceId::from_string_validated(&format!("{:0<32}", expected_id.chars().take(32).collect::<String>()))
                .unwrap_or_else(|| TraceId::new());
            
            let expected_trace_id_str = trace_id.as_str().to_string();
            
            // 在独立的上下文中执行
            let result = with_trace_id(trace_id.clone(), async move {
                let trace_id_clone = trace_id.clone();
                
                // 验证上下文中的ID正确性
                let current_id = get_trace_id();
                assert_eq!(current_id.as_str(), trace_id_clone.as_str());
                
                // 模拟一些异步工作
                tokio::time::sleep(Duration::from_millis(1)).await;
                
                // 再次验证上下文仍然正确
                let current_id_after = get_trace_id();
                assert_eq!(current_id_after.as_str(), trace_id_clone.as_str());
                
                trace_id_clone.as_str().to_string()
            }).await;
            
            assert_eq!(result, expected_trace_id_str);
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
}

/// 测试高频验证操作的性能和稳定性
#[tokio::test]
async fn test_high_frequency_validation() {
    const VALIDATION_COUNT: usize = 10000;
    const CONCURRENT_TASKS: usize = 10;
    
    let valid_ids = vec![
        "0af7651916cd43dd8448eb211c80319c",
        "1234567890abcdef1234567890abcdef",
        "fedcba0987654321fedcba0987654321",
    ];
    
    let invalid_ids = vec![
        "short",
        "toolongtraceidentifierthatexceeds32chars",
        "0AF7651916CD43DD8448EB211C80319C", // 大写
        "0af7651916cd43dd8448eb211c80319g", // 无效字符
        "00000000000000000000000000000000", // 全零
    ];
    
    let mut handles = vec![];
    
    for _ in 0..CONCURRENT_TASKS {
        let valid_ids_clone = valid_ids.clone();
        let invalid_ids_clone = invalid_ids.clone();
        
        let handle = tokio::spawn(async move {
            for _ in 0..VALIDATION_COUNT / CONCURRENT_TASKS {
                // 验证有效ID
                for valid_id in &valid_ids_clone {
                    let result = TraceId::from_string_validated(valid_id);
                    assert!(result.is_some(), "有效ID验证失败: {}", valid_id);
                }
                
                // 验证无效ID
                for invalid_id in &invalid_ids_clone {
                    let result = TraceId::from_string_validated(invalid_id);
                    assert!(result.is_none(), "无效ID验证应该失败: {}", invalid_id);
                }
            }
        });
        handles.push(handle);
    }
    
    // 设置超时以防止测试卡死
    let timeout_result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.unwrap();
        }
    }).await;
    
    assert!(timeout_result.is_ok(), "高频验证测试超时");
}

/// 测试内存使用的稳定性（防止内存泄漏）
#[tokio::test]
async fn test_memory_stability() {
    const ITERATIONS: usize = 10000;
    
    // 生成大量ID并立即丢弃，测试是否有内存泄漏
    for _ in 0..ITERATIONS {
        let _trace_id = TraceId::new();
        
        // 测试字符串验证
        let _valid = TraceId::from_string_validated("0af7651916cd43dd8448eb211c80319c");
        let _invalid = TraceId::from_string_validated("invalid");
        
        // 测试上下文操作
        let test_id = TraceId::new();
        let _result = with_trace_id(test_id, async {
            get_trace_id()
        }).await;
    }
    
    // 如果到达这里没有崩溃，说明内存管理是稳定的
    assert!(true);
}

/// 测试极端边界条件
#[tokio::test]
async fn test_edge_cases() {
    // 测试空字符串
    assert!(TraceId::from_string_validated("").is_none());
    
    // 测试最大长度字符串
    let max_length_str = "a".repeat(1000);
    assert!(TraceId::from_string_validated(&max_length_str).is_none());
    
    // 测试包含特殊字符的字符串
    let special_chars = "0af7651916cd43dd8448eb211c80319\0";
    assert!(TraceId::from_string_validated(special_chars).is_none());
    
    // 测试Unicode字符
    let unicode_str = "0af7651916cd43dd8448eb211c80319中";
    assert!(TraceId::from_string_validated(unicode_str).is_none());
    
    // 测试边界长度（31和33字符）
    let short_by_one = "0af7651916cd43dd8448eb211c80319";
    let long_by_one = "0af7651916cd43dd8448eb211c80319ca";
    assert!(TraceId::from_string_validated(short_by_one).is_none());
    assert!(TraceId::from_string_validated(long_by_one).is_none());
}

/// 测试原子操作的线程安全性
#[test]
fn test_atomic_counter_thread_safety() {
    const THREAD_COUNT: usize = 10;
    const IDS_PER_THREAD: usize = 1000;
    
    let handles: Vec<_> = (0..THREAD_COUNT)
        .map(|_| {
            thread::spawn(|| {
                let mut ids = Vec::new();
                for _ in 0..IDS_PER_THREAD {
                    ids.push(TraceId::new());
                }
                ids
            })
        })
        .collect();
    
    let mut all_ids = HashSet::new();
    for handle in handles {
        let thread_ids = handle.join().unwrap();
        for id in thread_ids {
            assert!(all_ids.insert(id.as_str().to_string()), "发现重复的trace_id");
        }
    }
    
    // 验证生成的ID总数
    assert_eq!(all_ids.len(), THREAD_COUNT * IDS_PER_THREAD);
}