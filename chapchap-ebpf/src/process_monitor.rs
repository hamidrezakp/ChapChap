use crate::vmlinux;

use aya_bpf::{
    macros::{lsm, map},
    maps::{HashMap, PerCpuArray},
    programs::LsmContext,
};
use aya_log_ebpf::debug;
use chapchap_common::process_monitor::{FILES_BLACKLIST_LEN, MAX_FILENAME};

pub(super) type LSMAction = i32;

#[repr(C)]
pub struct Buf {
    pub buf: [u8; MAX_FILENAME],
}

#[map]
pub static mut BUF: PerCpuArray<Buf> = PerCpuArray::with_max_entries(1, 0);

#[map]
pub static FILES_BLACKLIST: HashMap<u64, u8> = HashMap::with_max_entries(FILES_BLACKLIST_LEN, 0); //TODO: make len configurable

#[lsm(name = "process_monitor")]
pub fn process_monitor(ctx: LsmContext) -> LSMAction {
    match unsafe { try_process_monitor(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

pub(super) unsafe fn try_process_monitor(ctx: LsmContext) -> Result<LSMAction, LSMAction> {
    let binprm: *const vmlinux::linux_binprm = ctx.arg(0);

    let inode_number: u64 = (*(*(*binprm).file).f_inode).i_ino;

    debug!(&ctx, "inode number: {}", inode_number);

    if FILES_BLACKLIST.get(&inode_number).is_some() {
        return Ok(-1);
    } else {
        Ok(0)
    }
}
