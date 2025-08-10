//! TraceId 核心结构体定义

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 高性能追踪ID生成器
///
/// 使用时间戳 + 原子计数器的组合，生成符合W3C TraceContext规范的128位ID
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 机器ID，基于进程ID和启动时间戳生成，确保不同进程/实例的ID不冲突
static MACHINE_ID: LazyLock<u16> = LazyLock::new(|| {
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    ((pid ^ timestamp) & 0xFFFF) as u16
});

/// 追踪ID结构体
///
/// 支持多种ID格式：高性能模式使用时间戳+计数器，兼容模式使用UUID v4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceId(String);

impl TraceId {
    /// 获取机器ID
    ///
    /// 返回基于进程ID和启动时间戳生成的机器标识符
    #[inline]
    fn get_machine_id() -> u16 {
        *MACHINE_ID
    }

    /// 生成新的追踪ID（符合 W3C TraceContext 规范）
    ///
    /// 使用时间戳+计数器+机器ID+随机数的组合，生成32字符的小写十六进制ID
    /// 格式：符合 W3C TraceContext 规范的 trace-id
    /// 长度：固定32字符（128位）
    ///
    /// # 性能优化
    /// - 使用内联函数减少调用开销
    /// - 直接位操作避免额外计算
    /// - LazyLock确保机器ID初始化的线程安全
    ///
    /// # 返回
    /// 新生成的追踪ID
    #[inline]
    pub fn new() -> Self {
        // 获取当前时间戳（毫秒级）
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let machine_id = Self::get_machine_id();

        // 构造128位ID：timestamp(48位) + machine_id(16位) + counter(32位) + random(32位)
        let random_part = fastrand::u32(..);

        // 将各部分组合成128位数据
        let high_64 = ((timestamp & 0xFFFFFFFFFFFF) << 16) | (machine_id as u64);
        let low_64 = (counter & 0xFFFFFFFF) << 32 | (random_part as u64);

        // 转换为32字符的小写十六进制字符串
        let id = format!("{high_64:016x}{low_64:016x}");
        Self(id)
    }

    /// 从字符串创建追踪ID，并进行 W3C TraceContext 规范校验
    ///
    /// 高性能验证逻辑，使用字节级操作避免Unicode处理开销
    ///
    /// # 参数
    /// * `id` - 追踪ID字符串
    ///
    /// # 返回
    /// 如果格式有效则返回Some(TraceId)，否则返回None
    ///
    /// # 校验规则（符合 W3C TraceContext 规范）
    /// - 长度必须是 32 个字符
    /// - 只能包含小写十六进制字符（0-9, a-f）
    /// - 不能全为零（00000000000000000000000000000000）
    ///
    /// # 性能优化
    /// - 使用字节级验证避免Unicode处理
    /// - 提前返回优化分支预测
    /// - 内联函数减少调用开销
    #[inline]
    pub fn from_string_validated(id: &str) -> Option<Self> {
        // 长度检查：必须是 32 个字符
        if id.len() != 32 {
            return None;
        }

        // 字符有效性检查：使用字节级验证，性能更优
        if !Self::is_valid_hex_bytes(id.as_bytes()) {
            return None;
        }

        // 不能全为零
        if id == "00000000000000000000000000000000" {
            return None;
        }

        Some(Self(id.to_string()))
    }

    /// 高性能字节级十六进制字符验证
    ///
    /// 使用字节比较避免Unicode处理开销
    ///
    /// # 参数
    /// * `bytes` - 要验证的字节数组
    ///
    /// # 返回
    /// 如果所有字节都是小写十六进制字符则返回true
    #[inline]
    fn is_valid_hex_bytes(bytes: &[u8]) -> bool {
        bytes
            .iter()
            .all(|&b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    }

    /// 从字符串创建追踪ID（不进行校验，用于内部使用）
    ///
    /// 当确定输入有效时使用，避免额外的验证开销
    ///
    /// # 参数
    /// * `id` - 追踪ID字符串（假设已经过验证）
    ///
    /// # 返回
    /// TraceId实例
    ///
    /// # Safety
    /// 调用者需要确保输入字符串是有效的追踪ID格式
    ///
    /// # 性能优化
    /// - 内联函数减少调用开销
    /// - 跳过所有验证步骤
    #[inline]
    pub fn from_string_unchecked(id: &str) -> Self {
        Self(id.to_string())
    }

    /// 从字符串创建追踪ID
    ///
    /// 仅用于测试，不进行格式验证。
    #[cfg(test)]
    pub(crate) fn from_string(id: &str) -> Self {
        Self(id.to_string())
    }

    /// 获取追踪ID字符串
    ///
    /// # 返回
    /// 追踪ID的字符串表示
    ///
    /// # 性能优化
    /// - 内联函数减少调用开销
    /// - 直接返回内部字符串引用
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_creation() {
        let trace_id = TraceId::new();
        let id_str = trace_id.as_str();

        // 验证长度：必须是 32 个字符
        assert_eq!(id_str.len(), 32);

        // 验证只包含小写十六进制字符
        assert!(id_str
            .chars()
            .all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_lowercase())));

        // 验证不全为零
        assert_ne!(id_str, "00000000000000000000000000000000");
    }

    #[test]
    fn test_trace_id_from_string() {
        let id_str = "0af7651916cd43dd8448eb211c80319c";
        let trace_id = TraceId::from_string(id_str);
        assert_eq!(trace_id.as_str(), id_str);
    }

    #[test]
    fn test_trace_id_display() {
        let trace_id = TraceId::from_string("0af7651916cd43dd8448eb211c80319c");
        assert_eq!(format!("{}", trace_id), "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_trace_id_debug() {
        let trace_id = TraceId::from_string("0af7651916cd43dd8448eb211c80319c");
        let debug_str = format!("{:?}", trace_id);
        assert!(debug_str.contains("0af7651916cd43dd8448eb211c80319c"));
    }

    #[test]
    fn test_from_string_validated() {
        // Valid case: 符合 W3C TraceContext 规范的 trace-id
        let valid_id = "0af7651916cd43dd8448eb211c80319c";
        assert_eq!(
            TraceId::from_string_validated(valid_id),
            Some(TraceId(valid_id.to_string()))
        );

        // Invalid case: 长度不正确
        assert_eq!(TraceId::from_string_validated("short"), None);
        assert_eq!(
            TraceId::from_string_validated("toolongtraceidentifierthatexceeds32chars"),
            None
        );

        // Invalid case: 包含大写字符
        assert_eq!(
            TraceId::from_string_validated("0AF7651916CD43DD8448EB211C80319C"),
            None
        );

        // Invalid case: 包含非十六进制字符
        assert_eq!(
            TraceId::from_string_validated("0af7651916cd43dd8448eb211c80319g"),
            None
        );

        // Invalid case: 全为零
        assert_eq!(
            TraceId::from_string_validated("00000000000000000000000000000000"),
            None
        );
    }

    #[test]
    fn test_trace_id_uniqueness() {
        // 测试生成的ID的唯一性
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let trace_id = TraceId::new();
            assert!(
                ids.insert(trace_id.as_str().to_string()),
                "Generated duplicate trace ID"
            );
        }
    }

    #[test]
    fn test_w3c_traceparent_compatibility() {
        // 测试生成的ID是否可以用于构造 W3C traceparent header
        let trace_id = TraceId::new();
        let parent_id = "b7ad6b7169203331";
        let trace_flags = "01";

        let traceparent = format!("00-{}-{}-{}", trace_id.as_str(), parent_id, trace_flags);

        // 验证 traceparent 格式
        let parts: Vec<&str> = traceparent.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "00"); // version
        assert_eq!(parts[1].len(), 32); // trace-id
        assert_eq!(parts[2].len(), 16); // parent-id
        assert_eq!(parts[3].len(), 2); // trace-flags
    }

    #[test]
    fn test_additional_impls() {
        // 测试 Default trait
        let default_id = TraceId::default();
        assert_eq!(default_id.as_str().len(), 32);
        assert!(TraceId::from_string_validated(default_id.as_str()).is_some());

        // 测试 Clone 和 PartialEq traits
        let id1 = TraceId::new();
        let id2 = id1.clone();
        let id3 = TraceId::new();
        assert_eq!(id1, id2, "Cloned ID should be equal to the original");
        assert_ne!(id1, id3, "Different IDs should not be equal");

        // 测试 from_string_unchecked
        // 这个函数应该不进行任何验证，直接创建实例
        let invalid_str = "this-is-not-a-valid-id";
        let unchecked_id = TraceId::from_string_unchecked(invalid_str);
        assert_eq!(unchecked_id.as_str(), invalid_str);
        // 确认 from_string_validated 会拒绝这个ID
        assert!(TraceId::from_string_validated(invalid_str).is_none());
    }
}
