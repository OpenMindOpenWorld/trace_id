//! 核心功能性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trace_id::TraceId;

/// 基准测试：ID 生成
fn bench_id_generation(c: &mut Criterion) {
    c.bench_function("TraceId::new", |b| {
        b.iter(|| {
            // 使用 black_box 防止编译器优化掉ID的创建
            black_box(TraceId::new());
        })
    });
}

/// 基准测试：ID 验证
fn bench_id_validation(c: &mut Criterion) {
    let valid_id = "0af7651916cd43dd8448eb211c80319c";
    let invalid_id_length = "invalid-trace-id-that-is-longer";
    let invalid_id_chars = "0af7651916cd43dd8448eb211c80319g"; // 长度正确，但包含'g'
    let invalid_id_zero = "00000000000000000000000000000000"; // 全零ID

    let mut group = c.benchmark_group("TraceId::from_string_validated");

    // 测试有效ID的验证性能
    group.bench_function("valid_id", |b| {
        b.iter(|| {
            // 使用 black_box 防止编译器优化掉验证调用
            black_box(TraceId::from_string_validated(black_box(valid_id)));
        })
    });

    // 测试因长度错误而失败的性能
    group.bench_function("invalid_length", |b| {
        b.iter(|| {
            black_box(TraceId::from_string_validated(black_box(invalid_id_length)));
        })
    });

    // 新增：测试因无效字符而失败的性能
    group.bench_function("invalid_chars", |b| {
        b.iter(|| {
            black_box(TraceId::from_string_validated(black_box(invalid_id_chars)));
        })
    });

    // 新增：测试因全零而失败的性能
    group.bench_function("all_zeros", |b| {
        b.iter(|| {
            black_box(TraceId::from_string_validated(black_box(invalid_id_zero)));
        })
    });

    group.finish();
}

// 注册基准测试组
criterion_group!(benches, bench_id_generation, bench_id_validation);

// 运行基准测试
criterion_main!(benches);
