use crate::vmlinux;

use aya_bpf::{
    macros::{lsm, map},
    maps::HashMap,
    programs::LsmContext,
};

pub(super) type LSMAction = i32;

#[map]
pub static FILES_BLACKLIST: HashMap<u64, u8> = HashMap::with_max_entries(1024, 0); //TODO: make len configurable

#[lsm(name = "program_monitor")]
pub fn program_monitor(ctx: LsmContext) -> LSMAction {
    match unsafe { try_program_monitor(ctx) } {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

pub(super) unsafe fn try_program_monitor(ctx: LsmContext) -> Result<LSMAction, LSMAction> {
    let binprm: *const vmlinux::linux_binprm = ctx.arg(0);

    let inode_number: u64 = (*(*(*binprm).file).f_inode).i_ino;

    if FILES_BLACKLIST.get(&inode_number).is_some() {
        return Ok(-1);
    } else {
        Ok(0)
    }
}
