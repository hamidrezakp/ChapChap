use anyhow::Context;
use async_trait::async_trait;
use chapchap_common::rule_manager::{dbus_types, Rule, RuleID, RuleWithID};
use dyn_clonable::clonable;
use log::trace;
use mailbox_processor::{callback::CallbackMailboxProcessor, ReplyChannel};
use tokio::sync::OnceCell;
use zbus::{dbus_interface, Connection, ConnectionBuilder, SignalContext};

use self::types::Error;
use crate::rule_manager::RuleManager;

pub mod client;
mod types;

#[async_trait]
#[clonable]
pub trait IPCManager: Clone + Send + Sync {
    async fn emit_rules_changed(&self) -> Result<(), Error>;

    async fn stop(&self) -> Result<(), Error>;
}

struct State {
    rule_manager: Box<dyn RuleManager>,
}

#[derive(Debug)]
enum Message {
    AddRule(Rule, ReplyChannel<Result<RuleID, Error>>),
    DisableRule(RuleID, ReplyChannel<Result<(), Error>>),
    EnableRule(RuleID, ReplyChannel<Result<(), Error>>),

    GetRules(ReplyChannel<Result<Vec<Rule>, Error>>),
}

#[derive(Clone)]
pub struct IPCManagerImpl {
    mailbox: CallbackMailboxProcessor<Message>,

    dbus_connection: Option<Connection>,
    signal_context: OnceCell<SignalContext<'static>>,
}

#[async_trait]
impl IPCManager for IPCManagerImpl {
    async fn emit_rules_changed(&self) -> Result<(), Error> {
        if let Some(signal_context) = self.signal_context.get() {
            self.rules_changed(&signal_context)
                .await
                .map_err(|e| Error::Internal(format!("dbus signal error: {e:?}")))
        } else {
            Err(Error::Internal(
                "dbus connection/signal_context not ready".into(),
            ))
        }
    }

    async fn stop(&self) -> Result<(), Error> {
        self.mailbox.clone().stop().await;
        Ok(())
    }
}

async fn mailbox_step(_mb: CallbackMailboxProcessor<Message>, msg: Message, state: State) -> State {
    trace!("Got message: {msg:?}");

    match msg {
        Message::AddRule(rule, reply) => {
            let rule: Rule = rule.into();

            let result = state
                .rule_manager
                .add_rule(rule)
                .await
                .map_err(|e| Error::RuleManager(e));

            reply.reply(result)
        }
        Message::DisableRule(rule_id, reply) => {
            let result = state
                .rule_manager
                .disable_rule(rule_id)
                .await
                .map_err(|e| Error::RuleManager(e));

            reply.reply(result)
        }
        Message::EnableRule(rule_id, reply) => {
            let result = state
                .rule_manager
                .enable_rule(rule_id)
                .await
                .map_err(|e| Error::RuleManager(e));

            reply.reply(result)
        }
        Message::GetRules(reply) => {
            let result = state
                .rule_manager
                .get_rules()
                .await
                .map_err(|e| Error::RuleManager(e));
            reply.reply(result)
        }
    }
    state
}

#[dbus_interface(name = "ir.hrkp.Chapchap1.RuleManager")]
impl IPCManagerImpl {
    /// Add new rule
    async fn add_rule(&self, rule: dbus_types::Rule) -> zbus::fdo::Result<RuleID> {
        let rule: Rule = rule.try_into()?;

        trace!("Add rule: {rule:?}");

        let result = self
            .mailbox
            .post_and_reply(|r| Message::AddRule(rule, r))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()));

        result
    }

    /// Disable rule
    async fn disable_rule(&self, rule_id: dbus_types::RuleID) -> zbus::fdo::Result<()> {
        trace!("Disable rule: {rule_id}");

        self.mailbox
            .post_and_reply(|r| Message::DisableRule(rule_id, r))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Enable rule
    async fn enable_rule(&self, rule_id: dbus_types::RuleID) -> zbus::fdo::Result<()> {
        trace!("Enable rule: {rule_id}");

        self.mailbox
            .post_and_reply(|r| Message::EnableRule(rule_id, r))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    #[dbus_interface(property)]
    async fn rules(&self) -> zbus::fdo::Result<Vec<dbus_types::Rule>> {
        trace!("Getting all the rule");

        let rules = self
            .mailbox
            .post_and_reply(|r| Message::GetRules(r))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
            .unwrap()
            .map(|v| v.into_iter().map(Into::into).collect::<Vec<_>>())
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
            .unwrap();

        trace!("Rules: {rules:?}");
        Ok(rules)
    }
}

pub async fn start(rule_manager: Box<dyn RuleManager>) -> anyhow::Result<Box<dyn IPCManager>> {
    let state = State { rule_manager };

    let mut ipc = IPCManagerImpl {
        mailbox: CallbackMailboxProcessor::start(mailbox_step, state, 1000),
        dbus_connection: None,
        signal_context: OnceCell::new(),
    };

    let serving_path = "/ir/hrkp/Chapchap1/RuleManager".to_string();

    let dbus_connection = ConnectionBuilder::session()?
        .name("ir.hrkp.Chapchap")?
        .serve_at(serving_path.clone(), ipc.clone())?
        .build()
        .await?;

    ipc.dbus_connection = Some(dbus_connection.clone());

    let signal_context =
        SignalContext::new(&dbus_connection, serving_path).context("creating signal context")?;
    ipc.signal_context
        .set(signal_context)
        .context("setting signal context")?;

    Ok(Box::new(ipc))
}
