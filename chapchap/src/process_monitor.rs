use aya::{maps::HashMap, programs::Lsm, Bpf, Btf};

pub(super) fn setup(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
    let btf = Btf::from_sys_fs()?;
    let program_monitor: &mut Lsm = bpf.program_mut("process_monitor").unwrap().try_into()?;
    program_monitor.load("bprm_check_security", &btf)?;
    program_monitor.attach()?;

    let mut blacklist: HashMap<_, u64, u8> =
        HashMap::try_from(bpf.take_map("FILES_BLACKLIST").unwrap())?;

    blacklist.insert(4890656, 0, 0).unwrap();

    Ok(())
}
