use super::{
    dbus_types::{self, RuleWithID},
    Error, RuleID,
};
use zbus::dbus_proxy;

#[dbus_proxy(
    default_service = "ir.hrkp.Chapchap",
    interface = "ir.hrkp.Chapchap1.RuleManager",
    default_path = "/ir/hrkp/Chapchap1",
    gen_blocking = false
)]
trait Rules {
    async fn add_rule(&self, rule: dbus_types::Rule) -> Result<RuleID, Error>;
    async fn disable_rule(&self, rule_id: dbus_types::RuleID) -> Result<(), Error>;
    async fn enable_rule(&self, rule_id: dbus_types::RuleID) -> Result<(), Error>;

    #[dbus_proxy(property)]
    fn rules(&self) -> zbus::fdo::Result<Vec<RuleWithID>>;
}
