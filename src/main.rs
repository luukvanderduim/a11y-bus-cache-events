use std::error::Error;

use atspi::zbus::MatchRule;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn Error>> {
    // set a11y on session bus
    atspi::set_session_accessibility(true).await?;

    if let Err(e) = atspi::Connection::open().await {
        panic!("Could not connect to Registry daemon: {e:?}");
    }
    let registry = atspi::Connection::open().await?;

    if let Err(e) = registry.register_event("Cache:AddAccessible").await {
        panic!("Could not register event: {e:?}");
    }

    if let Err(e) = registry.register_event("Cache:RemoveAccessible").await {
        panic!("Could not register event: {e:?}");
    }

    let rule_add = MatchRule::builder()
        .interface("org.a11y.atspi.Cache")?
        .member("AddAccessible")?
        .build();
    let rule_rem = MatchRule::builder()
        .interface("org.a11y.atspi.Cache")?
        .member("RemoveAccessible")?
        .build();

    let a11y_bus = atspi::Connection::open().await?;
    let a11y_dbus_connection = a11y_bus.inner().connection();

    // For the FreeDesktop Org. primary bus fnctionality
    let dbus_proxy = atspi::zbus::fdo::DBusProxy::new(a11y_dbus_connection).await?;

    dbus_proxy.add_match_rule(rule_add).await?;
    dbus_proxy.add_match_rule(rule_rem).await?;

    let registry_deamon_stream = registry.event_stream();

    tokio::pin!(registry_deamon_stream);

    while let Some(ev) = registry_deamon_stream.next().await {
        match ev {
            Ok(ev) => println!("My Precious: {}", ev.event_string()),
            _ => println!("Error on stream -- "),
        };
    }

    Ok(())
}
