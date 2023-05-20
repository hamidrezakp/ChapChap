use anyhow::Context;
use ebpf_manager::EBPFManager;
use tokio::signal;

mod ebpf_manager;
mod ipc_manager;
mod rule_manager;

pub async fn service() -> anyhow::Result<()> {
    let (rule_manager, mut rule_manager_notification) = rule_manager::start()?;
    let ebpf_manager = ebpf_manager::start()?;
    let (ipc_manager, _ipc_manager_notification, _conn) =
        ipc_manager::start(rule_manager.clone()).await?;

    ebpf_manager
        .process_monitor_load()
        .await
        .context("Load program monitor")?;

    loop {
        tokio::select! {
            sig = signal::ctrl_c() =>
                match sig {
                    Ok(_) => {
                        println!("got ctrl_c");
                        ebpf_manager.stop().await?; //TODO: aggregate errors rather than short-circuiting
                        rule_manager.stop().await?;
                        ipc_manager.stop().await?;
                        break Ok(());
                    }
                    Err(e) => eprintln!("Error while receiving signal: {e}"),
                },

            Some(notification) = rule_manager_notification.recv() => {
                match notification {
                    rule_manager::Notification::RuleAdded{rule, ..} => {
                        if !rule.is_active {
                            continue;
                        }

                        match rule.module {
                            chapchap_common::types::Module::ProgramMonitor(module_rule) =>
                                handle_program_monitor_rule_addition(&ebpf_manager, module_rule).await,
                        }
                    },

                    rule_manager::Notification::RuleRemoved{rule, ..} =>
                    {
                        if !rule.is_active {
                            continue;
                        }

                        match rule.module {
                            chapchap_common::types::Module::ProgramMonitor(module_rule) =>
                                handle_program_monitor_rule_deletion(&ebpf_manager, module_rule).await,
                        }
                    },

                    rule_manager::Notification::RuleUpdated{..} => todo!(),
                }
            }
        }
    }
}

async fn handle_program_monitor_rule_addition(
    ebpf_manager: &Box<dyn EBPFManager>,
    rule: chapchap_common::types::program_monitor::Rule,
) {
    match rule.filter {
        chapchap_common::types::program_monitor::Filter::Basic => match rule.action {
            chapchap_common::types::program_monitor::Action::BlockProgramExecution(inode) => {
                if let Err(e) = ebpf_manager.process_monitor_block_program(inode).await {
                    log::error!("Failed to block program: {e}");
                }
            }
        },
        _ => unimplemented!(),
    }
}

async fn handle_program_monitor_rule_deletion(
    ebpf_manager: &Box<dyn EBPFManager>,
    rule: chapchap_common::types::program_monitor::Rule,
) {
    match rule.filter {
        chapchap_common::types::program_monitor::Filter::Basic => match rule.action {
            chapchap_common::types::program_monitor::Action::BlockProgramExecution(inode) => {
                if let Err(e) = ebpf_manager.process_monitor_allow_program(inode).await {
                    log::error!("Failed to allow program: {e}");
                }
            }
        },
        _ => unimplemented!(),
    }
}
