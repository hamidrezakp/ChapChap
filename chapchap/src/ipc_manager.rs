use async_trait::async_trait;
use chapchap_common::{
    dbus_types,
    types::{Rule, RuleID},
};
use dyn_clonable::clonable;
use mailbox_processor::{callback::CallbackMailboxProcessor, notification_channel, ReplyChannel};
use zbus::{dbus_interface, Connection, ConnectionBuilder};

use self::types::Error;
use crate::rule_manager::RuleManager;

mod types;

pub const SERVICE_BASE_ID: &str = "ir.hrkp.Chapchap";

#[async_trait]
#[clonable]
pub trait IPCManager: Clone + Send + Sync {
    async fn stop(&self) -> Result<(), Error>;
}

struct State {
    rule_manager: Box<dyn RuleManager>,

    notification_tx: notification_channel::Sender<Notification>,
}

impl State {
    pub fn notify(&self, notification: Notification) {
        self.notification_tx.send(notification)
    }
}

enum Message {
    AddRule(Rule, ReplyChannel<Result<RuleID, Error>>),
}

#[derive(Clone)]
pub struct IPCManagerImpl {
    mailbox: CallbackMailboxProcessor<Message>,
}

#[async_trait]
impl IPCManager for IPCManagerImpl {
    async fn stop(&self) -> Result<(), Error> {
        self.mailbox.clone().stop().await;
        Ok(())
    }
}

async fn mailbox_step(_mb: CallbackMailboxProcessor<Message>, msg: Message, state: State) -> State {
    match msg {
        Message::AddRule(rule, reply) => {
            let rule: dbus_types::Rule = rule.into();
        }
    }
    state
}

pub enum Notification {}

#[dbus_interface(name = "ir.hrkp.Chapchap1")]
impl IPCManagerImpl {
    /// Add new rule
    async fn add_rule(&self, rule: dbus_types::Rule) -> Result<RuleID, dbus_types::Error> {
        let rule: Rule = rule.try_into()?;
        self.mailbox
            .post_and_reply(|r| Message::AddRule(rule, r))
            .await
            .map_err(|_| dbus_types::Error::Internal)?
            .map_err(Into::into)
    }
}

pub async fn start(
    rule_manager: Box<dyn RuleManager>,
) -> anyhow::Result<(
    Box<dyn IPCManager>,
    notification_channel::Receiver<Notification>,
    Connection,
)> {
    let (notification_tx, notification_rx) = notification_channel::channel();

    let state = State {
        notification_tx,
        rule_manager,
    };

    let ipc = IPCManagerImpl {
        mailbox: CallbackMailboxProcessor::start(mailbox_step, state, 1000),
    };

    let conn = ConnectionBuilder::session()?
        .name(SERVICE_BASE_ID)?
        .serve_at("/ir/hrkp/Chapchap", ipc.clone())?
        .build()
        .await?;

    Ok((Box::new(ipc), notification_rx, conn))
}
