# support_hashmap 实现总结

## 题目要求

在 `no_std` 环境下支持 `HashMap`，测试验证能够插入 50,000 个键值对并正确遍历。

## 问题分析

`std::collections::HashMap` 依赖标准库的哈希实现，在 `no_std` 环境不可用。
需要使用 `hashbrown` crate 提供的 `HashMap`。

## 实现方案

### 修改文件

1. `arceos/ulib/axstd/Cargo.toml` - 添加 hashbrown 依赖
2. `arceos/ulib/axstd/src/lib.rs` - 导出 HashMap

### Cargo.toml 修改

```toml
[dependencies]
hashbrown = { version = "0.15", optional = true }

[features]
alloc = ["arceos_api/alloc", "axfeat/alloc", "axio/alloc", "hashbrown"]
```

### lib.rs 修改

```rust
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::{boxed, format, string, vec};

// 提供 HashMap 通过 hashbrown
#[cfg(feature = "alloc")]
pub mod collections {
    pub use hashbrown::HashMap;
}
```

### 使用示例

```rust
use std::collections::HashMap;

fn test_hashmap() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();
    for value in 0..N {
        let key = format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u32>().unwrap(), *v);
        }
    }
}
```

### 关键点

1. **可选依赖**: hashbrown 作为可选依赖，只在启用 `alloc` feature 时引入
2. **避免冲突**: 不重用 `alloc::collections`，而是创建新的 `collections` 模块
3. **版本固定**: 使用 `indexmap 2.6.0` 避免 Rust 2024 edition 问题

## 测试结果

```
Running memory tests...
test_hashmap() OK!
Memory tests run OK!
support_hashmap pass
```

## 依赖关系

```
axstd
    ├── alloc (feature) ── hashbrown
    └── collections::HashMap
```

## 为什么选择 hashbrown

- 纯 `no_std` 实现
- 高性能（Rust 标准库内部也使用）
- API 与标准库兼容
- 不依赖系统调用