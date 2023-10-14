//! # DBus interface proxy for: `org.freedesktop.Application`

use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.Application",
    assume_defaults = false,
    gen_blocking = false
)]
trait Application {
    /// Activate method
    fn activate(
        &self,
        platform_data: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    /// ActivateAction method
    fn activate_action(
        &self,
        action_name: &str,
        parameter: &[zbus::zvariant::Value<'_>],
        platform_data: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    /// Open method
    fn open(
        &self,
        uris: &[&str],
        platform_data: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;
}
