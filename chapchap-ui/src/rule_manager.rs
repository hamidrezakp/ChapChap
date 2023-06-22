use async_trait::async_trait;
use chapchap_common::rule_manager::{client::RulesProxy, dbus_types, Rule, RuleID, RuleWithID};
use dyn_clonable::clonable;
use zbus::{export::futures_util::TryFutureExt, zvariant::Value, Connection};

use self::types::Error;

mod types;

#[async_trait]
#[clonable]
pub trait RuleManager: Clone + Send + Sync {
    async fn add_rule(&self, rule: Rule) -> Result<RuleID, Error>;
    async fn disable_rule(&self, rule_id: RuleID) -> Result<(), Error>;
    async fn enable_rule(&self, rule_id: RuleID) -> Result<(), Error>;

    async fn rules(&self) -> Result<Vec<RuleWithID>, Error>;
    async fn stop(&self) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct RuleManagerImpl<'a> {
    rule_proxy: RulesProxy<'a>,
}

#[async_trait]
impl<'a> RuleManager for RuleManagerImpl<'a> {
    async fn add_rule(&self, rule: Rule) -> Result<RuleID, Error> {
        let rule: dbus_types::Rule = rule.into();
        self.rule_proxy.add_rule(rule).map_err(Into::into).await
    }

    async fn disable_rule(&self, rule_id: RuleID) -> Result<(), Error> {
        self.rule_proxy
            .disable_rule(rule_id)
            .map_err(Into::into)
            .await
    }

    async fn enable_rule(&self, rule_id: RuleID) -> Result<(), Error> {
        self.rule_proxy
            .enable_rule(rule_id)
            .map_err(Into::into)
            .await
    }

    async fn rules(&self) -> Result<Vec<RuleWithID>, Error> {
        let rules: Vec<dbus_types::RuleWithID> = self
            .rule_proxy
            .rules()
            .await
            .map_err(|e| Error::DBus(e.to_string()))?
            .try_into()
            .map_err(|_| Error::DBus("invalid type".to_string()))?;

        rules
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<RuleWithID>, _>>()
            .map_err(|_| Error::DBus("invalid type".to_string()))
    }

    async fn stop(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub async fn start() -> anyhow::Result<Box<dyn RuleManager>> {
    let conn = Connection::session().await?;

    let rule_manager = RuleManagerImpl {
        rule_proxy: RulesProxy::new(&conn).await?,
    };

    Ok(Box::new(rule_manager))
}
