use async_trait::async_trait;
use chapchap_common::rule_manager::{
    client::RuleManagerProxy, dbus_types, Rule, RuleID, RuleWithID,
};
use dyn_clonable::clonable;
use zbus::Connection;

use self::types::Error;

mod types;

#[async_trait]
#[clonable]
pub trait RuleManager: Clone + Send + Sync {
    async fn add_rule(&self, rule: Rule) -> zbus::Result<RuleID>;
    async fn disable_rule(&self, rule_id: RuleID) -> zbus::Result<()>;
    async fn enable_rule(&self, rule_id: RuleID) -> zbus::Result<()>;

    async fn rules(&self) -> zbus::Result<Vec<RuleWithID>>;
    async fn stop(&self) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct RuleManagerImpl<'a> {
    rule_proxy: RuleManagerProxy<'a>,
}

#[async_trait]
impl<'a> RuleManager for RuleManagerImpl<'a> {
    async fn add_rule(&self, rule: Rule) -> zbus::Result<RuleID> {
        let rule: dbus_types::Rule = rule.into();
        self.rule_proxy.add_rule(rule).await
    }

    async fn disable_rule(&self, rule_id: RuleID) -> zbus::Result<()> {
        self.rule_proxy.disable_rule(rule_id).await
    }

    async fn enable_rule(&self, rule_id: RuleID) -> zbus::Result<()> {
        self.rule_proxy.enable_rule(rule_id).await
    }

    async fn rules(&self) -> zbus::Result<Vec<RuleWithID>> {
        let rules: Vec<dbus_types::RuleWithID> = self.rule_proxy.rules().await?;

        Ok(rules
            .into_iter()
            .map(|i| i.try_into().unwrap()) //TODO
            .collect::<Vec<RuleWithID>>())
    }

    async fn stop(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub async fn start() -> anyhow::Result<Box<dyn RuleManager>> {
    let conn = Connection::session().await?;

    let rule_manager = RuleManagerImpl {
        rule_proxy: RuleManagerProxy::new(&conn).await?,
    };

    Ok(Box::new(rule_manager))
}
