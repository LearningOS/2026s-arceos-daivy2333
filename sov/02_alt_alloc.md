# alt_alloc 实现总结

## 题目要求

实现一个 bump allocator（早期内存分配器），支持：
- 字节分配（向前增长）
- 页分配（向后增长）
- 双端内存管理

测试验证：能够分配 3,000,000 个元素并进行排序。

## 实现方案

**修改文件**: `arceos/modules/bump_allocator/src/lib.rs`

### 内存布局

```
[ bytes-used | avail-area | pages-used ]
|            | -->    <-- |            |
start       b_pos        p_pos       end
```

### 核心结构

```rust
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,     // 内存起始
    end: usize,       // 内存结束
    b_pos: usize,     // 字节分配位置（向前）
    p_pos: usize,     // 页分配位置（向后）
    count: usize,     // 活动字节分配计数
}
```

### 关键实现

#### 字节分配 (ByteAllocator)

```rust
fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
    let size = layout.size();
    let align = layout.align();
    
    // 对齐 b_pos
    let aligned_pos = (self.b_pos + align - 1) & !(align - 1);
    
    // 检查空间是否足够
    if aligned_pos + size > self.p_pos {
        return Err(AllocError::NoMemory);
    }
    
    // 分配并更新位置
    self.b_pos = aligned_pos + size;
    self.count += 1;
    
    Ok(NonNull::new(aligned_pos as *mut u8).unwrap())
}
```

#### 页分配 (PageAllocator)

```rust
fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
    let size = num_pages * Self::PAGE_SIZE;
    let align = 1 << align_pow2;
    
    // 向后对齐 p_pos
    let aligned_pos = self.p_pos & !(align - 1);
    
    // 检查空间
    if aligned_pos - size < self.b_pos {
        return Err(AllocError::NoMemory);
    }
    
    // 向后分配
    self.p_pos = aligned_pos - size;
    Ok(self.p_pos)
}
```

### 设计要点

1. **双端分配**: 字节向前，页向后，避免冲突
2. **计数释放**: `count` 记录分配次数，归零时重置字节区域
3. **对齐处理**: 正确处理 layout 的对齐要求
4. **空间检查**: 分配前检查是否有足够空间

## 测试结果

```
Running bump tests...
Bump tests run OK!
alt_alloc pass
```

## 依赖关系

```
alt_axalloc (全局分配器包装)
    └── bump_allocator (EarlyAllocator 实现)
            └── allocator trait (接口定义)
```