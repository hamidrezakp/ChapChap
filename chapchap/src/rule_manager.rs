use std::collections::{hash_map, HashMap};

use async_trait::async_trait;
use chapchap_common::types::{Rule, RuleID};
use dyn_clonable::clonable;
use mailbox_processor::{callback::CallbackMailboxProcessor, notification_channel, ReplyChannel};

pub use self::types::Error;
use self::types::Result;

mod types;

#[async_trait]
#[clonable]
pub trait RuleManager: Clone + Send + Sync {
    async fn add_rule(&self, rule: Rule) -> Result<RuleID>;
    async fn remove_rule(&self, rule_id: RuleID);
    async fn update_rule(&self, rule_id: RuleID, new_rule: Rule) -> Result<()>;

    async fn disable_rule(&self, rule_id: RuleID) -> Result<()>;
    async fn enable_rule(&self, rule_id: RuleID) -> Result<()>;

    async fn stop(&self) -> Result<()>;
}

struct State {
    next_rule_id: RuleID,
    rules: HashMap<RuleID, Rule>,

    notification_tx: notification_channel::Sender<Notification>,
}

impl State {
    pub fn next_rule_id(&mut self) -> RuleID {
        let rule_id = self.next_rule_id;
        self.next_rule_id += 1;
        rule_id
    }

    pub fn notify(&self, notification: Notification) {
        self.notification_tx.send(notification)
    }
}

enum Message {
    AddRule(Rule, ReplyChannel<Result<RuleID>>),
    RemoveRule(RuleID),
    UpdateRule(RuleID, Rule, ReplyChannel<Result<()>>),
    EnableRule(RuleID, ReplyChannel<Result<()>>),
    DisableRule(RuleID, ReplyChannel<Result<()>>),
}

#[derive(Clone)]
pub struct RuleManagerImpl {
    mailbox: CallbackMailboxProcessor<Message>,
}

#[async_trait]
impl RuleManager for RuleManagerImpl {
    async fn add_rule(&self, rule: Rule) -> Result<RuleID> {
        self.mailbox
            .post_and_reply(|r| Message::AddRule(rule, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn remove_rule(&self, rule_id: RuleID) {
        self.mailbox.post_and_forget(Message::RemoveRule(rule_id));
    }

    async fn update_rule(&self, rule_id: RuleID, rule: Rule) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::UpdateRule(rule_id, rule, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn disable_rule(&self, rule_id: RuleID) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::DisableRule(rule_id, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn enable_rule(&self, rule_id: RuleID) -> Result<()> {
        self.mailbox
            .post_and_reply(|r| Message::EnableRule(rule_id, r))
            .await
            .map_err(|e| Error::Internal(format!("MailboxError: {e}")))?
    }

    async fn stop(&self) -> Result<()> {
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
        Message::AddRule(rule, reply) => {
            let result = if let Some((rule_id, _)) = state.rules.iter().find(|(_, v)| **v == rule) {
                Err(Error::RuleAlreadyExist(*rule_id))
            } else {
                let rule_id = state.next_rule_id();
                state.rules.insert(rule_id, rule.clone());
                state.notify(Notification::RuleAdded { rule_id, rule });
                Ok(rule_id)
            };
            reply.reply(result);
        }

        Message::RemoveRule(rule_id) => {
            if let Some(rule) = state.rules.remove(&rule_id) {
                state.notify(Notification::RuleRemoved { rule_id, rule });
            }
        }

        Message::UpdateRule(rule_id, new_rule, reply) => {
            let result = match state.rules.entry(rule_id) {
                hash_map::Entry::Occupied(mut occ) => {
                    let old_rule = occ.insert(new_rule.clone());
                    state.notify(Notification::RuleUpdated {
                        rule_id,
                        old_rule,
                        new_rule,
                    });
                    Ok(())
                }
                hash_map::Entry::Vacant(_) => Err(Error::RuleNotFound),
            };
            reply.reply(result);
        }

        Message::EnableRule(rule_id, reply) => {
            let result = match state.rules.entry(rule_id) {
                hash_map::Entry::Occupied(mut occ) => {
                    let old_rule = occ.get().clone();
                    occ.get_mut().is_active = true;
                    let new_rule = occ.get().clone();

                    state.notify(Notification::RuleUpdated {
                        rule_id,
                        old_rule,
                        new_rule,
                    });

                    Ok(())
                }

                hash_map::Entry::Vacant(_) => Err(Error::RuleNotFound),
            };
            reply.reply(result);
        }
        Message::DisableRule(rule_id, reply) => {
            let result = match state.rules.entry(rule_id) {
                hash_map::Entry::Occupied(mut occ) => {
                    let old_rule = occ.get().clone();
                    occ.get_mut().is_active = false;
                    let new_rule = occ.get().clone();

                    state.notify(Notification::RuleUpdated {
                        rule_id,
                        old_rule,
                        new_rule,
                    });

                    Ok(())
                }

                hash_map::Entry::Vacant(_) => Err(Error::RuleNotFound),
            };
            reply.reply(result);
        }
    }
    state
}

pub enum Notification {
    RuleAdded {
        rule_id: RuleID,
        rule: Rule,
    },
    RuleRemoved {
        rule_id: RuleID,
        rule: Rule,
    },
    RuleUpdated {
        rule_id: RuleID,
        old_rule: Rule,
        new_rule: Rule,
    },
}

pub fn start() -> Result<(
    Box<dyn RuleManager>,
    notification_channel::Receiver<Notification>,
)> {
    let (notification_tx, notification_rx) = notification_channel::channel();

    let state = State {
        next_rule_id: 0,
        rules: HashMap::<RuleID, Rule>::new(),
        notification_tx,
    };

    Ok((
        Box::new(RuleManagerImpl {
            mailbox: CallbackMailboxProcessor::start(mailbox_step, state, 1000),
        }),
        notification_rx,
    ))
}
