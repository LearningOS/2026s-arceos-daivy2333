# ramfs_rename 实现总结

## 题目要求

在 RAM 文件系统中实现文件重命名功能（`rename` 操作）。

测试验证：
- 创建文件 `/tmp/f1` 并写入内容
- 重命名为 `/tmp/f2`
- 能够读取重命名后的文件内容

## 问题分析

`axfs_vfs::VfsNodeOps` trait 有 `rename` 方法，默认返回 `Unsupported`。
需要在 `axfs_ramfs::DirNode` 中实现该方法。

## 实现方案

**修改文件**: `arceos/axfs_ramfs/src/dir.rs`

### 核心实现

```rust
fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
    log::debug!("rename at ramfs: {} -> {}", src_path, dst_path);

    // 提取目标路径的最终文件名（处理完整路径如 /tmp/f2）
    let dst_final_name = dst_path
        .trim_start_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(dst_path);

    let (src_name, src_rest) = split_path(src_path);

    // 如果源路径有子路径组件，递归处理
    if let Some(rest) = src_rest {
        let subdir = self
            .children
            .read()
            .get(src_name)
            .ok_or(VfsError::NotFound)?
            .clone();
        return subdir.rename(rest, dst_path);
    }

    // 同目录重命名
    if src_name.is_empty() || src_name == "." || src_name == ".." {
        return Err(VfsError::InvalidInput);
    }
    if dst_final_name.is_empty() || dst_final_name == "." || dst_final_name == ".." {
        return Err(VfsError::InvalidInput);
    }

    // 执行重命名：移除旧条目，插入新条目
    let mut children = self.children.write();
    let src_node = children.get(src_name).ok_or(VfsError::NotFound)?.clone();
    children.remove(src_name);
    children.insert(dst_final_name.into(), src_node);

    Ok(())
}
```

### 关键设计

1. **路径解析**: 正确处理完整路径 `/tmp/f2`，提取最终文件名 `f2`
2. **递归处理**: 支持嵌套目录路径
3. **锁机制**: 使用 `RwLock` 保护 children BTreeMap
4. **边界检查**: 禁止重命名 `.` 和 `..`

### Cargo 配置

修改 `arceos/Cargo.toml`:

```toml
[workspace.dependencies]
axfs_ramfs = { path = "./axfs_ramfs" }

[patch.crates-io]
axfs_ramfs = { path = "./axfs_ramfs" }
```

修改 `arceos/axfs_ramfs/Cargo.toml` 版本为 `0.1.2` 以匹配 patch。

## 测试结果

```
Create '/tmp/f1' and write [hello] ...
Rename '/tmp/f1' to '/tmp/f2' ...
Read '/tmp/f2' content: [hello] ok!
[Ramfs-Rename]: ok!
ramfs_rename pass
```

## VFS 层次结构

```
RootDirectory (axfs/src/root.rs)
    └── VfsNodeOps::rename()
            └── lookup_mounted_fs()
                    └── RamFileSystem (axfs_ramfs)
                            └── DirNode::rename()
```

## 文件系统挂载

```
/          → fatfs (主文件系统)
/tmp       → ramfs (内存文件系统)
/dev       → devfs (设备文件系统)
/proc      → ramfs (进程信息)
/sys       → ramfs (系统信息)
```