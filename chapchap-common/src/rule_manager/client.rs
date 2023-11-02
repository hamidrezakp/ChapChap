use super::{
    dbus_types::{self, RuleWithID},
    RuleID,
};
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "ir.hrkp.Chapchap1.RuleManager",
    default_service = "ir.hrkp.Chapchap",
    default_path = "/ir/hrkp/Chapchap1/RuleManager",
    gen_blocking = false
)]
trait RuleManager {
    /// AddRule method
    async fn add_rule(&self, rule: dbus_types::Rule) -> zbus::Result<RuleID>;

    /// DisableRule method
    async fn disable_rule(&self, rule_id: dbus_types::RuleID) -> zbus::Result<()>;

    /// EnableRule method
    async fn enable_rule(&self, rule_id: dbus_types::RuleID) -> zbus::Result<()>;

    /// Rules property
    #[dbus_proxy(property)]
    fn rules(&self) -> zbus::Result<Vec<RuleWithID>>;
}
