use std::sync::Arc;

use aya::{
    maps::{HashMap, MapData},
    programs::Lsm,
    Bpf, Btf,
};
use chapchap_common::rule_manager::program_monitor::INodeNumber;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub struct ProgramMonitor {
    bpf: Arc<Mutex<Bpf>>,
    files_blacklist: HashMap<MapData, INodeNumber, u8>,
}

impl ProgramMonitor {
    pub async fn load(bpf: Arc<Mutex<Bpf>>) -> Result<Self> {
        let mut bpf_handle = bpf.lock().await;
        let btf = Btf::from_sys_fs().map_err(|e| Error::Btf(e.to_string()))?;

        let program = Self::program(&mut bpf_handle)?;

        program
            .load("bprm_check_security", &btf)
            .map_err(|e| Error::Program(e.to_string()))?;

        program
            .attach()
            .map_err(|e| Error::Program(e.to_string()))?;
        log::info!("program monitor attached");

        let files_blacklist = HashMap::try_from(
            bpf_handle
                .take_map("FILES_BLACKLIST")
                .ok_or_else(|| Error::MapNotFound("FILES_BLACKLIST".to_string()))?,
        )
        .map_err(|e| Error::Map(e.to_string()))?;

        drop(bpf_handle);

        Ok(Self {
            files_blacklist,
            bpf,
        })
    }

    pub async fn unload(self) -> Result<()> {
        let mut bpf = self.bpf.lock().await;
        Self::program(&mut bpf)?
            .unload()
            .map_err(|e| Error::Program(e.to_string()))
    }

    pub async fn block_program(&mut self, inode: INodeNumber) -> Result<()> {
        self.files_blacklist
            .insert(inode, 0, 0)
            .map_err(|e| Error::Map(e.to_string()))
    }

    pub async fn allow_program(&mut self, inode: INodeNumber) -> Result<()> {
        self.files_blacklist
            .remove(&inode)
            .map_err(|e| Error::Map(e.to_string()))
    }

    fn program<'a>(bpf: &'a mut Bpf) -> Result<&'a mut Lsm> {
        Ok(bpf
            .program_mut("program_monitor")
            .ok_or_else(|| Error::ProgramNotFound)?
            .try_into()
            .unwrap()) //TODO
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
