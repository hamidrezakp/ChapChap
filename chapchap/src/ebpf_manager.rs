use async_trait::async_trait;
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use chapchap_common::types::program_monitor::INodeNumber;
use dyn_clonable::clonable;
use log::warn;
use mailbox_processor::{callback::CallbackMailboxProcessor, ReplyChannel};

use self::modules::program_monitor::ProgramMonitor;
use self::types::{Error, Result};

mod modules;
mod types;

#[async_trait]
#[clonable]
pub trait EBPFManager: Clone + Send + Sync {
    async fn process_monitor_load(&self) -> Result<()>;
    async fn process_monitor_unload(&self) -> Result<()>;
    async fn process_monitor_allow_program(&self, inode: INodeNumber) -> Result<()>;
    async fn process_monitor_block_program(&self, inode: INodeNumber) -> Result<()>;

    async fn stop(&self) -> Result<()>;
}

struct State {
    bpf: Bpf,

    process_monitor: ProgramMonitor,
}

enum Message {
    ProcessMonitorLoad(ReplyChannel<Result<()>>),
    ProcessMonitorUnload(ReplyChannel<Result<()>>),
    ProcessMonitorAllowProgram(INodeNumber, ReplyChannel<Result<()>>),
    ProcessMonitorBlockProgram(INodeNumber, ReplyChannel<Result<()>>),
}

#[derive(Clone)]
pub struct EBPFManagerImpl {
    mailbox: CallbackMailboxProcessor<Message>,
}

#[async_trait]
impl EBPFManager for EBPFManagerImpl {
    async fn process_monitor_load(&self) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProcessMonitorLoad(r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn process_monitor_unload(&self) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProcessMonitorUnload(r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn process_monitor_allow_program(&self, inode: INodeNumber) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProcessMonitorAllowProgram(inode, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn process_monitor_block_program(&self, inode: INodeNumber) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProcessMonitorBlockProgram(inode, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn stop(&self) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::ProcessMonitorUnload(r))
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
        Message::ProcessMonitorLoad(reply) => {
            let result = state
                .process_monitor
                .load(&mut state.bpf)
                .map_err(Error::ProcessMonitor);

            reply.reply(result);
        }
        Message::ProcessMonitorUnload(reply) => {
            let result = state
                .process_monitor
                .unload(&mut state.bpf)
                .map_err(Error::ProcessMonitor);

            reply.reply(result);
        }

        Message::ProcessMonitorAllowProgram(inode, reply) => {
            let result = state
                .process_monitor
                .allow_program(&mut state.bpf, inode)
                .map_err(Error::ProcessMonitor);

            reply.reply(result);
        }
        Message::ProcessMonitorBlockProgram(inode, reply) => {
            let result = state
                .process_monitor
                .block_program(&mut state.bpf, inode)
                .map_err(Error::ProcessMonitor);

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
        bpf,
        process_monitor: ProgramMonitor,
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
