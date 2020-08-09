//! 文件相关的内核功能

use super::*;
use crate::fs::ROOT_INODE;
use core::slice;
use core::slice::from_raw_parts_mut;
use core::str;

/// 从指定的文件中读取字符
///
/// 如果缓冲区暂无数据，返回 0；出现错误返回 -1
pub(super) fn sys_read(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    // 从进程中获取 inode
    let process = PROCESSOR.lock().current_thread().process.clone();
    if let Some(inode) = process.inner().descriptors.get(fd) {
        // 从系统调用传入的参数生成缓冲区
        let buffer = unsafe { from_raw_parts_mut(buffer, size) };
        // 尝试读取
        if let Ok(ret) = inode.read_at(0, buffer) {
            let ret = ret as isize;
            if ret > 0 {
                return SyscallResult::Proceed(ret);
            }
            if ret == 0 {
                return SyscallResult::Park(ret);
            }
        }
    }
    SyscallResult::Proceed(-1)
}

/// 将字符写入指定的文件
pub(super) fn sys_write(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    // 从进程中获取 inode
    let process = PROCESSOR.lock().current_thread().process.clone();
    if let Some(inode) = process.inner().descriptors.get(fd) {
        // 从系统调用传入的参数生成缓冲区
        let buffer = unsafe { from_raw_parts_mut(buffer, size) };
        // 尝试写入
        if let Ok(ret) = inode.write_at(0, buffer) {
            let ret = ret as isize;
            if ret >= 0 {
                return SyscallResult::Proceed(ret);
            }
        }
    }
    SyscallResult::Proceed(-1)
}

pub(super) fn sys_open(buffer: *mut u8, size: usize) -> SyscallResult {
    let name = unsafe {
        let slice = slice::from_raw_parts(buffer, size);
        str::from_utf8(slice).unwrap()
    };
    // 从文件系统中找到程序
    let file = ROOT_INODE.find(name).unwrap();
    let process = PROCESSOR.lock().current_thread().process.clone();
    process.inner().descriptors.push(file);
    SyscallResult::Proceed(
        (PROCESSOR
            .lock()
            .current_thread()
            .process
            .clone()
            .inner()
            .descriptors
            .len()
            - 1) as isize,
    )
}
