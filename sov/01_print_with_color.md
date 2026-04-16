# print_with_color 实现总结

## 题目要求

实现彩色输出功能，在终端打印带有 ANSI 颜色代码的文本。

测试验证：
- 输出包含 ANSI 颜色代码 (`\x1b[`)
- 输出包含文本 "Hello, Arceos!"

## 实现方案

**修改文件**: `arceos/ulib/axstd/src/macros.rs`

### 核心代码

```rust
/// ANSI color codes
pub mod color {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const RESET: &str = "\x1b[0m";
}

/// Prints to the standard output, with a newline (with color).
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("{}\x1b[32m{}\x1b[0m\n", format_args!($($arg)*)));
    }
}
```

### 关键点

1. **ANSI 颜色码**: 使用 `\x1b[32m` 表示绿色，`\x1b[0m` 重置颜色
2. **宏修改**: 在 `println!` 宏中嵌入颜色代码，使所有输出默认带颜色
3. **不修改 main.rs**: 按题目要求，只在 `macros.rs` 中实现

## 测试结果

```
[32m[WithColor]: Hello, Arceos![0m
Has color
Has Hello, Arceos!
print_with_color pass
```

## ANSI 颜色码参考

| 颜色 | 代码 |
|------|------|
| 黑色 | `\x1b[30m` |
| 红色 | `\x1b[31m` |
| 绿色 | `\x1b[32m` |
| 黄色 | `\x1b[33m` |
| 蓝色 | `\x1b[34m` |
| 紫色 | `\x1b[35m` |
| 青色 | `\x1b[36m` |
| 白色 | `\x1b[37m` |
| 重置 | `\x1b[0m` |