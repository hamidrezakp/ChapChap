use anyhow::Context;
use ebpf_manager::EBPFManager;
use log::trace;
use tokio::signal;

mod ebpf_manager;
mod ipc_manager;
mod rule_manager;

pub async fn service() -> anyhow::Result<()> {
    console_subscriber::init();

    trace!("Setting up services");

    let (rule_manager, mut rule_manager_notification) = rule_manager::start()?;
    let ebpf_manager = ebpf_manager::start()?;
    let ipc_manager = ipc_manager::start(rule_manager.clone()).await?;

    trace!("All services are up");

    ebpf_manager
        .program_monitor_load()
        .await
        .context("Load program monitor")?;

    trace!("porgram monitor loaded");

    loop {
        tokio::select! {
            sig = signal::ctrl_c() =>
                match sig {
                    Ok(_) => {
                        println!("got ctrl_c, stopping now ...");
                        ipc_manager.stop().await?; //TODO: aggregate errors rather than short-circuiting
                        ebpf_manager.stop().await?;
                        rule_manager.stop().await?;
                        break Ok(());
                    }
                    Err(e) => eprintln!("Error while receiving signal: {e}"),
                },

            Some(notification) = rule_manager_notification.recv() => {
                match notification {
                    rule_manager::Notification::RuleAdded{rule, ..} => {
                        ipc_manager.rules_changed().await?;

                        if !rule.is_active {
                            continue;
                        }

                        match rule.module {
                            chapchap_common::rule_manager::Module::ProgramMonitor(module_rule) =>
                                add_program_monitor_rule(&ebpf_manager, module_rule).await,
                        }
                    },

                    rule_manager::Notification::RuleRemoved{rule, ..} =>
                    {
                        ipc_manager.rules_changed().await?;

                        if !rule.is_active {
                            continue;
                        }

                        match rule.module {
                            chapchap_common::rule_manager::Module::ProgramMonitor(module_rule) =>
                                remove_program_monitor_rule(&ebpf_manager, module_rule).await,
                        }
                    },

                    rule_manager::Notification::RuleUpdated{old_rule, new_rule, ..} => {
                        ipc_manager.rules_changed().await?;

                        if !old_rule.is_active && new_rule.is_active {
                            match new_rule.module {
                                chapchap_common::rule_manager::Module::ProgramMonitor(module_rule) =>
                                    add_program_monitor_rule(&ebpf_manager, module_rule).await,
                            }
                        } else if old_rule.is_active && !new_rule.is_active {
                            match new_rule.module {
                                chapchap_common::rule_manager::Module::ProgramMonitor(module_rule) =>
                                    remove_program_monitor_rule(&ebpf_manager, module_rule).await,
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn add_program_monitor_rule(
    ebpf_manager: &Box<dyn EBPFManager>,
    rule: chapchap_common::rule_manager::program_monitor::Rule,
) {
    match rule.filter {
        chapchap_common::rule_manager::program_monitor::Filter::Basic => match rule.action {
            chapchap_common::rule_manager::program_monitor::Action::BlockProgramExecution(
                inode,
            ) => {
                if let Err(e) = ebpf_manager.program_monitor_block_program(inode).await {
                    log::error!("Failed to block program: {e}");
                }
            }
        },
        _ => unimplemented!(),
    }
}

async fn remove_program_monitor_rule(
    ebpf_manager: &Box<dyn EBPFManager>,
    rule: chapchap_common::rule_manager::program_monitor::Rule,
) {
    match rule.filter {
        chapchap_common::rule_manager::program_monitor::Filter::Basic => match rule.action {
            chapchap_common::rule_manager::program_monitor::Action::BlockProgramExecution(
                inode,
            ) => {
                if let Err(e) = ebpf_manager.program_monitor_allow_program(inode).await {
                    log::error!("Failed to allow program: {e}");
                }
            }
        },
        _ => unimplemented!(),
    }
}
