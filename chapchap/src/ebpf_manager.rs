use std::sync::Arc;

use async_trait::async_trait;
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use chapchap_common::rule_manager::program_monitor::INodeNumber;
use dyn_clonable::clonable;
use log::{trace, warn};
use mailbox_processor::{callback::CallbackMailboxProcessor, ReplyChannel};
use tokio::sync::Mutex;

use self::modules::program_monitor::ProgramMonitor;
use self::types::{Error, Result};

mod modules;
mod types;

#[async_trait]
#[clonable]
pub trait EBPFManager: Clone + Send + Sync {
    async fn program_monitor_load(&self) -> Result<()>;
    async fn program_monitor_unload(&self) -> Result<()>;
    async fn program_monitor_allow_program(&self, inode: INodeNumber) -> Result<()>;
    async fn program_monitor_block_program(&self, inode: INodeNumber) -> Result<()>;

    async fn stop(&self) -> Result<()>;
}

struct State {
    bpf: Arc<Mutex<Bpf>>,

    program_monitor: Option<ProgramMonitor>,
}

enum Message {
    ProgramMonitorLoad(ReplyChannel<Result<()>>),
    ProgramMonitorUnload(ReplyChannel<Result<()>>),
    ProgramMonitorAllowProgram(INodeNumber, ReplyChannel<Result<()>>),
    ProgramMonitorBlockProgram(INodeNumber, ReplyChannel<Result<()>>),
}

#[derive(Clone)]
pub struct EBPFManagerImpl {
    mailbox: CallbackMailboxProcessor<Message>,
}

#[async_trait]
impl EBPFManager for EBPFManagerImpl {
    async fn program_monitor_load(&self) -> Result<()> {
        trace!("[program monitor] Loading");

        self.mailbox
            .post_and_reply(|r| Message::ProgramMonitorLoad(r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn program_monitor_unload(&self) -> Result<()> {
        trace!("[program monitor] Unloading");

        self.mailbox
            .post_and_reply(|r| Message::ProgramMonitorUnload(r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn program_monitor_allow_program(&self, inode: INodeNumber) -> Result<()> {
        trace!("[program monitor] Allow: {inode}");

        self.mailbox
            .post_and_reply(|r| Message::ProgramMonitorAllowProgram(inode, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn program_monitor_block_program(&self, inode: INodeNumber) -> Result<()> {
        trace!("[program monitor] Block: {inode}");

        self.mailbox
            .post_and_reply(|r| Message::ProgramMonitorBlockProgram(inode, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn stop(&self) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProgramMonitorUnload(r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))??;

        self.mailbox.clone().stop().await;
        Ok(())
    }
}

async fn mailbox_step(
    _mb: CallbackMailboxProcessor<Message>,
    msg: Message,
    mut state: State,
) -> State {
    match msg {
        Message::ProgramMonitorLoad(reply) => {
            let result = match state.program_monitor {
                Some(_) => Err(Error::ModuleAlreadyLoaded("program_monitor")),
                None => match ProgramMonitor::load(state.bpf.clone()).await {
                    Ok(p) => {
                        state.program_monitor = Some(p);
                        Ok(())
                    }
                    Err(e) => Err(Error::ProgramMonitor(e)),
                },
            };

            reply.reply(result);
        }
        Message::ProgramMonitorUnload(reply) => {
            let result = if state.program_monitor.is_none() {
                Err(Error::ModuleNotLoaded("program_monitor"))
            } else {
                let program_monitor = state.program_monitor.take().unwrap();

                match program_monitor.unload().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::ProgramMonitor(e)),
                }
            };

            reply.reply(result);
        }

        Message::ProgramMonitorAllowProgram(inode, reply) => {
            let result = match state.program_monitor {
                Some(ref mut p) => p.allow_program(inode).await.map_err(Error::ProgramMonitor),
                None => Err(Error::ModuleNotLoaded("program_monitor")),
            };

            reply.reply(result);
        }
        Message::ProgramMonitorBlockProgram(inode, reply) => {
            let result = match state.program_monitor {
                Some(ref mut p) => p.block_program(inode).await.map_err(Error::ProgramMonitor),
                None => Err(Error::ModuleNotLoaded("program_monitor")),
            };

            reply.reply(result);
        }
    }
    state
}

pub fn start() -> Result<Box<dyn EBPFManager>> {
    let mut bpf = get_ebpf_module()?;

    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let state = State {
        bpf: Arc::new(Mutex::new(bpf)),
        program_monitor: None,
    };

    Ok(Box::new(EBPFManagerImpl {
        mailbox: CallbackMailboxProcessor::start(mailbox_step, state, 1000),
    }))
}

fn get_ebpf_module() -> Result<Bpf> {
    #[cfg(debug_assertions)]
    {
        Bpf::load(include_bytes_aligned!(
            "../../target/bpfel-unknown-none/debug/chapchap"
        ))
        .map_err(|e| Error::Btf(e.to_string()))
    }

    #[cfg(not(debug_assertions))]
    {
        Bpf::load(include_bytes_aligned!(
            "../../target/bpfel-unknown-none/release/chapchap"
        ))
        .map_err(|e| Error::Btf(e.to_string()))
    }
}
