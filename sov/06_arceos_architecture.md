# ArceOS 架构设计分析

## 架构概览

ArceOS 采用**模块化、层次化、组件化**的设计理念。每个功能模块都是独立的 crate，通过 trait 定义接口，实现松耦合、高可配置性。

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                          │
│         (exercises/print_with_color, ramfs_rename, etc.)     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      axstd / axlibc                          │
│            (Standard Library / POSIX API Layer)              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │
│  │ macros  │ │   io    │ │   fs    │ │  sync   │            │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     arceos_api                               │
│               (System Call Interface Layer)                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │
│  │  alloc  │ │   fs    │ │  task   │ │  net    │            │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       axfeat                                │
│              (Feature Configuration Layer)                   │
│         Controls which modules are enabled                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Core Modules                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                     axruntime                         │  │
│  │              (Kernel Entry & Runtime)                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │   axhal    │ │    axmm    │ │   axtask   │              │
│  │ (Hardware) │ │ (Memory)   │ │ (Tasks)    │              │
│  └────────────┘ └────────────┘ └────────────┘              │
│                                                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │   axfs     │ │   axnet    │ │   axsync   │              │
│  │(Filesystem)│ │ (Network)  │ │(Synchronization)│         │
│  └────────────┘ └────────────┘ └────────────┘              │
│                                                              │
│  ┌────────────┐ ┌────────────┐                              │
│  │  axalloc   │ │  axdriver  │                              │
│  │(Allocator) │ │  (Drivers) │                              │
│  └────────────┘ └────────────┘                              │
│                                                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │  axlog     │ │  axconfig  │ │  axdisplay │              │
│  │  (Logging) │ │ (Config)   │ │ (Graphics) │              │
│  └────────────┘ └────────────┘ └────────────┘              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Hardware Platform                         │
│            (riscv64-qemu-virt, aarch64, x86_64)              │
└─────────────────────────────────────────────────────────────┘
```

## 层次详解

### 1. 用户应用层 (Applications)

位置: `arceos/exercises/`, `arceos/examples/`

特点:
- 每个应用是独立的 Cargo 项目
- 通过 `axstd` 或 `axlibc` 接口调用系统功能
- 编译为独立内核镜像

### 2. 标准库层 (axstd/axlibc)

位置: `arceos/ulib/axstd/`, `arceos/ulib/axlibc/`

```
axstd/
├── src/
│   ├── lib.rs      # 模块导出
│   ├── macros.rs   # println!, print! 等
│   ├── io/         # 输入输出
│   ├── fs/         # 文件系统操作
│   ├── sync/       # 同步原语
│   ├── thread/     # 线程管理
│   ├── net/        # 网络操作
│   └── collections/ # HashMap 等 (通过 hashbrown)
```

特点:
- 类似 Rust `std` 的接口
- 直接调用模块函数，无系统调用开销
- 可选功能通过 Cargo features 控制

### 3. API 层 (arceos_api)

位置: `arceos/api/arceos_api/`

```rust
// 示例: 文件系统 API
pub fn ax_open(path: &str) -> io::Result<FileRef> { ... }
pub fn ax_read(file: &FileRef, buf: &mut [u8]) -> io::Result<usize> { ... }
pub fn ax_rename(old: &str, new: &str) -> io::Result<()> { ... }
```

特点:
- 统一的系统调用接口
- 连接上层库和下层模块
- 错误类型转换

### 4. Feature 配置层 (axfeat)

位置: `arceos/api/axfeat/`

功能:
- 控制 enabled features
- 初始化各模块
- 配置编译选项

### 5. 核心模块层 (Modules)

位置: `arceos/modules/`

#### axhal - 硬件抽象层

```
axhal/
├── src/
│   ├── arch/       # 架构特定代码
│   │   ├── riscv64/
│   │   ├── aarch64/
│   │   └── x86_64/
│   ├── mem.rs      # 内存地址类型
│   ├── paging.rs   # 页表操作
│   └── platform/   # 平台特定代码
```

职责:
- CPU 架构抽象
- 平台初始化
- 中断处理
- 上下文切换

#### axmm - 内存管理

```
axmm/
├── src/
│   ├── lib.rs      # 地址空间管理
│   └── mapping.rs  # 内存映射
```

职责:
- 虚拟地址空间
- 页表管理
- 内存分配/释放

#### axalloc - 内存分配器

```
axalloc/
├── src/
│   ├── lib.rs      # 全局分配器
│   ├── byte.rs     # 字节分配器
│   └── page.rs     # 页分配器

alt_axalloc/         # 备选分配器
├── src/
│   └── lib.rs       # 使用 bump_allocator
```

分配器选择:
- `alloc-tlsf`: TLSF 分配器
- `alloc-slab`: Slab 分配器
- `alloc-buddy`: Buddy 分配器
- `alt_alloc`: Bump 分配器 (我们实现)

#### axfs - 文件系统

```
axfs/
├── src/
│   ├── lib.rs      # 文件系统核心
│   ├── root.rs     # 根目录/挂载管理
│   ├── fops.rs     # 文件操作
│   ├── fs/
│   │   ├── fatfs.rs    # FAT 文件系统
│   │   ├── myfs.rs     # 自定义文件系统
│   │   └── mod.rs
│   └── api/
│       └── mod.rs      # 文件系统 API

axfs_ramfs/         # RAM 文件系统
├── src/
│   ├── lib.rs
│   ├── dir.rs      # 目录节点 (我们修改)
│   └── file.rs     # 文件节点
```

挂载结构:
```
/       → FATFS (磁盘)
/tmp    → RAMFS (内存)
/dev    → DEVFS (设备)
/proc   → RAMFS (进程信息)
/sys    → RAMFS (系统信息)
```

#### axtask - 任务管理

```
axtask/
├── src/
│   ├── lib.rs      # 任务核心
│   ├── task.rs     # 任务结构
│   ├── scheduler/  # 调度器
│   │   ├── fifo.rs
│   │   ├── rr.rs
│   │   └── cfs.rs
```

调度策略:
- `sched_fifo`: FIFO 协作调度
- `sched_rr`: Round-Robin 抢占调度
- `sched_cfs`: 完全公平调度

#### axsync - 同步原语

```
axsync/
├── src/
│   ├── lib.rs
│   ├── mutex.rs    # 互斥锁
│   ├── semaphore.rs # 信号量
│   └── condvar.rs  # 条件变量
```

#### axdriver - 设备驱动

```
axdriver/
├── src/
│   ├── lib.rs
│   ├── bus/        # 总线驱动
│   │   ├── pci.rs
│   │   ├── mmio.rs
│   ├── block/      # 块设备
│   ├── net/        # 网络设备
│   └── display/    # 显示设备
```

## 组件化设计特点

### 1. 功能即组件

每个模块是独立的 crate:
```
modules/
├── axalloc/        # 可以单独编译测试
├── axfs/           # 可以单独编译测试
├── axtask/         # 可以单独编译测试
...
```

好处:
- 模块独立测试
- 按需组合功能
- 减少编译时间

### 2. Feature Gate 控制

通过 Cargo features 精确控制功能:

```toml
[features]
# CPU
smp = ["axfeat/smp"]
fp_simd = ["axfeat/fp_simd"]

# Memory
alloc = ["axfeat/alloc", "axio/alloc"]
alloc-tlsf = ["axfeat/alloc-tlsf"]
alt_alloc = ["arceos_api/alt_alloc"]

# Task
multitask = ["axfeat/multitask"]
sched_rr = ["axfeat/sched_rr"]

# FS
fs = ["arceos_api/fs"]
myfs = ["arceos_api/myfs"]
```

编译示例:
```bash
# 最小内核 (无文件系统, 无网络)
make A=examples/helloworld/

# 带文件系统
make A=examples/shell/ FS=y

# 带网络
make A=examples/httpserver/ NET=y

# 带 alt_alloc
make A=exercises/alt_alloc/ alt_alloc=y
```

### 3. Trait 抽象

模块间通过 trait 定义接口:

```rust
// allocator trait
pub trait ByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>;
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout);
    fn total_bytes(&self) -> usize;
    fn used_bytes(&self) -> usize;
}

// VfsNodeOps trait
pub trait VfsNodeOps: Send + Sync {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr>;
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize>;
    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult;
    ...
}
```

### 4. 平台抽象

支持多架构:

```
axhal/src/
├── arch/
│   ├── riscv64/    # RISC-V 64位
│   ├── aarch64/    # ARM 64位
│   └── x86_64/     # x86 64位
└── platform/
    ├── qemu-virt/  # QEMU 虚拟平台
    └── real_hw/    # 真实硬件
```

编译命令:
```bash
make A=app PLATFORM=riscv64-qemu-virt
make A=app PLATFORM=aarch64-qemu-virt
make A=app PLATFORM=x86_64-qemu-virt
```

## 依赖关系图

```
┌──────────────────────────────────────────────────────────────┐
│                         Application                          │
└──────────────────────────────────────────────────────────────┘
            │                    │                    │
            ▼                    ▼                    ▼
    ┌───────────┐        ┌───────────┐        ┌───────────┐
    │   axstd   │        │  axlibc   │        │  axlog    │
    └───────────┘        └───────────┘        └───────────┘
            │                    │                    │
            └────────────────────┼────────────────────┘
                                 ▼
                        ┌───────────┐
                        │arceos_api │
                        └───────────┘
                                 │
            ┌────────────────────┼────────────────────┐
            │                    │                    │
            ▼                    ▼                    ▼
    ┌───────────┐        ┌───────────┐        ┌───────────┐
    │   axfs    │        │  axtask   │        │   axnet   │
    └───────────┘        └───────────┘        └───────────┘
            │                    │                    │
            │            ┌───────┴───────┐            │
            │            │               │            │
            ▼            ▼               ▼            ▼
    ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐
    │ axdriver  │ │   axmm    │ │  axsync   │ │  axhal    │
    └───────────┘ └───────────┘ └───────────┘ └───────────┘
            │            │                    │
            └────────────┼────────────────────┘
                         ▼
                ┌───────────┐
                │  axalloc  │
                └───────────┘
                         │
                         ▼
                ┌───────────┐
                │allocator* │ (TLSF/Slab/Buddy/Bump)
                └───────────┘
```

## 初始化流程

```rust
// axruntime/src/lib.rs
#[no_mangle]
pub extern "C" fn rust_main(cpu_id: usize, dtb: usize) {
    // 1. 硬件初始化
    axhal::init(cpu_id, dtb);
    
    // 2. 日志初始化
    axlog::init();
    
    // 3. 内存分配器初始化
    axalloc::init();
    
    // 4. 文件系统初始化
    #[cfg(feature = "fs")]
    axfs::init();
    
    // 5. 任务调度器初始化
    #[cfg(feature = "multitask")]
    axtask::init();
    
    // 6. 网络初始化
    #[cfg(feature = "net")]
    axnet::init();
    
    // 7. 运行主函数
    main();
    
    // 8. 结束处理
    axhal::terminate();
}
```

## 与传统内核对比

| 特点 | 传统内核 | ArceOS |
|------|---------|--------|
| 系统调用 | 用户态→内核态切换 | 直接函数调用 |
| 模块耦合 | 紧耦合 | 松耦合 |
| 功能裁剪 | 需修改源码 | Cargo features |
| 多架构支持 | 需大量适配 | trait 抽象 |
| 测试 | 需完整内核 | 模块独立测试 |
| 应用开发 | 用户态程序 | 内核态应用 |

## 设计哲学总结

1. **功能即组件**: 每个功能是独立 crate
2. **配置即编译**: 通过 features 控制功能
3. **接口即 trait**: 模块间 trait 定义接口
4. **应用即内核**: 应用直接编译为内核
5. **平台即抽象**: 硬件平台 trait 抽象

这种设计使得 ArceOS:
- 高度可定制
- 易于测试
- 易于移植
- 模块可替换
- 编译效率高