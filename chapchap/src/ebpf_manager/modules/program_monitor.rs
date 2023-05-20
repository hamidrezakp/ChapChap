use aya::{
    maps::{HashMap, MapData},
    programs::Lsm,
    Bpf, Btf,
};
use chapchap_common::types::program_monitor::INodeNumber;
use serde::{Deserialize, Serialize};

pub struct ProgramMonitor;

impl ProgramMonitor {
    pub fn load(&self, bpf: &mut Bpf) -> Result<()> {
        let btf = Btf::from_sys_fs().map_err(|e| Error::Btf(e.to_string()))?;

        let program = self.program(bpf)?;

        program
            .load("bprm_check_security", &btf)
            .map_err(|e| Error::Program(e.to_string()))?;

        program
            .attach()
            .map_err(|e| Error::Program(e.to_string()))?;
        log::info!("program monitor attached");

        Ok(())
    }

    pub fn unload(&self, bpf: &mut Bpf) -> Result<()> {
        self.program(bpf)?
            .unload()
            .map_err(|e| Error::Program(e.to_string()))
    }

    pub fn block_program(&self, bpf: &mut Bpf, inode: INodeNumber) -> Result<()> {
        self.blacklist(bpf)?
            .insert(inode, 0, 0)
            .map_err(|e| Error::Map(e.to_string()))
    }

    pub fn allow_program(&self, bpf: &mut Bpf, inode: INodeNumber) -> Result<()> {
        self.blacklist(bpf)?
            .remove(&inode)
            .map_err(|e| Error::Map(e.to_string()))
    }

    fn program<'a>(&self, bpf: &'a mut Bpf) -> Result<&'a mut Lsm> {
        Ok(bpf
            .program_mut("process_monitor") //TODO: change to program_monitor
            .ok_or_else(|| Error::ProgramNotFound)?
            .try_into()
            .unwrap()) //TODO
    }

    fn blacklist(&self, bpf: &mut Bpf) -> Result<HashMap<MapData, u64, u8>> {
        HashMap::try_from(
            bpf.take_map("FILES_BLACKLIST")
                .ok_or_else(|| Error::MapNotFound("FILES_BLACKLIST".to_string()))?,
        )
        .map_err(|e| Error::Map(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Btf: {0}")]
    Btf(String),

    #[error("Program not found")]
    ProgramNotFound,

    #[error("Program error: {0}")]
    Program(String),

    #[error("Map not found: {0}")]
    MapNotFound(String),

    #[error("Map error: {0}")]
    Map(String),
}

pub type Result<T> = std::result::Result<T, Error>;
