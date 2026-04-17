# sys_map 实现总结

## 题目要求

实现 `sys_mmap` 系统调用，支持用户程序通过内存映射读取文件内容。

测试验证：
- 用户程序创建文件写入 "hello, arceos!"
- 通过 `mmap` 映射文件到内存
- 打印 "Read back content: hello, arceos!"

## 问题分析

用户程序 `mapfile.c` 流程：
1. 创建文件 `test_file` 并写入内容
2. 打开文件，调用 `mmap(NULL, 32, PROT_READ, MAP_PRIVATE, fd, 0)`
3. 从映射的内存地址读取内容并打印

需要实现 `SYS_MMAP` (syscall 222) 系统调用。

## 实现方案

**修改文件**: `arceos/exercises/sys_map/src/syscall.rs`

### 核心实现

```rust
fn sys_mmap(
    addr: *mut usize,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    _offset: isize,
) -> isize {
    let mmap_prot = MmapProt::from_bits_truncate(prot);
    let mmap_flags = MmapFlags::from_bits_truncate(flags);

    let curr = axtask::current();
    let aspace = curr.task_ext().aspace.clone();
    let mut aspace = aspace.lock();

    // 转换权限标志
    let map_flags: MappingFlags = mmap_prot.into();

    // 页对齐长度
    let length_aligned = (length + PAGE_SIZE_4K - 1) & !(PAGE_SIZE_4K - 1);

    // 分配虚拟地址（addr 为 NULL 时由内核选择）
    let vaddr = if addr.is_null() {
        let hint = VirtAddr::from(0x100000usize);
        let limit = VirtAddrRange::from_start_size(VirtAddr::from(0), aspace.end().as_usize());
        match aspace.find_free_area(hint, length_aligned, limit) {
            Some(vaddr) => vaddr,
            None => return -LinuxError::ENOMEM.code() as _,
        }
    } else {
        VirtAddr::from(addr as usize).align_down_4k()
    };

    // 映射内存区域
    match aspace.map_alloc(vaddr, length_aligned, map_flags, true) {
        Ok(_) => {}
        Err(_) => return -LinuxError::ENOMEM.code() as _,
    }

    // 如果不是匿名映射且有有效 fd，读取文件内容到映射区域
    if !mmap_flags.contains(MmapFlags::MAP_ANONYMOUS) && fd >= 0 {
        let mut buf = vec![0u8; length];
        let ret = api::sys_read(fd, buf.as_mut_ptr() as *mut c_void, length);
        if ret > 0 {
            aspace.write(vaddr, &buf[..ret as usize]).ok();
        }
    }

    vaddr.as_usize() as isize
}
```

### 权限标志定义

```rust
bitflags::bitflags! {
    struct MmapProt: i32 {
        const PROT_READ = 1 << 0;
        const PROT_WRITE = 1 << 1;
        const PROT_EXEC = 1 << 2;
    }
}

impl From<MmapProt> for MappingFlags {
    fn from(value: MmapProt) -> Self {
        let mut flags = MappingFlags::USER;
        if value.contains(MmapProt::PROT_READ) {
            flags |= MappingFlags::READ;
        }
        if value.contains(MmapProt::PROT_WRITE) {
            flags |= MappingFlags::WRITE;
        }
        if value.contains(MmapProt::PROT_EXEC) {
            flags |= MappingFlags::EXECUTE;
        }
        flags
    }
}

bitflags::bitflags! {
    struct MmapFlags: i32 {
        const MAP_SHARED = 1 << 0;
        const MAP_PRIVATE = 1 << 1;
        const MAP_FIXED = 1 << 4;
        const MAP_ANONYMOUS = 1 << 5;
    }
}
```

### 关键设计点

1. **地址分配**: 使用 `find_free_area` 在用户地址空间找到空闲区域
2. **权限转换**: 将 Linux mmap 权限转换为 ArceOS MappingFlags
3. **文件映射**: 通过临时缓冲区读取文件，再用 `aspace.write()` 写入用户空间
4. **页对齐**: 确保地址和长度都页对齐

### 系统调用号映射

```rust
const SYS_MMAP: usize = 222;  // RISC-V Linux syscall number
```

## 测试结果

```
handle_syscall [222] ...
handle_syscall [66] ...
Read back content: hello, arceos!
handle_syscall [57] ...
handle_syscall [66] ...
MapFile ok!
handle_syscall [94] ...
[SYS_EXIT_GROUP]: system is exiting ..
monolithic kernel exit [Some(0)] normally!
sys_mmap pass
```

## 系统调用处理流程

```
用户程序 mmap()
    ↓
Trap: Supervisor ECALL
    ↓
TrapHandler::handle_syscall
    ↓
sys_mmap (syscall 222)
    ↓
┌─────────────────────────────┐
│ 1. 获取用户地址空间         │
│ 2. 查找空闲虚拟地址         │
│ 3. 映射内存区域             │
│ 4. 读取文件内容写入映射区   │
│ 5. 返回映射地址             │
└─────────────────────────────┘
    ↓
返回用户程序
```

## 用户态程序结构

用户程序 `mapfile.c` 编译为 RISC-V ELF：
- 使用 musl-gcc 静态编译
- 放入磁盘镜像 `/sbin/mapfile`
- 由内核加载执行

## 依赖模块

```
sys_map
├── axmm (地址空间管理)
│   ├── AddrSpace::find_free_area
│   ├── AddrSpace::map_alloc
│   └── AddrSpace::write
├── axhal (硬件抽象)
│   ├── paging::MappingFlags
│   └── mem::VirtAddr
├── axtask (任务管理)
│   ├── current()
│   └── TaskExtRef::aspace
└── arceos_posix_api (POSIX API)
    └── sys_read
```