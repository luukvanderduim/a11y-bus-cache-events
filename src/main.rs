use atspi::zbus::{
  MatchRule,
  MessageType,
};
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> Result<(), Box<dyn Error>> {
    // set a11y on session bus
    atspi::set_session_accessibility(true).await?;
    // Open connection
    let registry = atspi::Connection::open().await?;

    let caret_ev = "Object:TextCaretMoved:";

    let registry_evs = [
        "Registry:EventListenerRegistered",
        "Registry:EventListenerRegistered",
    ];
    let cache_evs = ["Cache:AddAccessible", "Cache:RemoveAccessible"];
    let socket_ev = "Socket::Available";

    for ev in registry_evs {
        if let Err(e) = registry.register_event(ev).await {
            panic!("Could not register event: {e:?}");
        }
    }
    for ev in cache_evs {
        if let Err(e) = registry.register_event(ev).await {
            panic!("Could not register event: {e:?}");
        }
    }
    if let Err(e) = registry.register_event(socket_ev).await {
        panic!("Could not register event: {e:?}");
    }
    if let Err(e) = registry.register_event(caret_ev).await {
        panic!("Could not register event: {e:?}");
    }

    let rule_rem = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Cache")?
        .member("RemoveAccessible")?
        .build();
    let rule_add = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Cache")?
        .member("AddAccessible")?
        .build();
    let rule_avail = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Socket")?
        .build();

    let rule_reg = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Registry")?
        .build();

    let rule_mse = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Object")?
        .member("TextCaretMoved")?
        .build();

    // For the FreeDesktop Org. primary bus fnctionality
    let dbus_proxy = atspi::zbus::fdo::DBusProxy::new(registry.connection()).await?;
    println!("DBus Proxy path: {}", dbus_proxy.path());

    dbus_proxy.add_match_rule(rule_add).await?;
    dbus_proxy.add_match_rule(rule_rem).await?;
    dbus_proxy.add_match_rule(rule_reg).await?;
    dbus_proxy.add_match_rule(rule_avail).await?;
    dbus_proxy.add_match_rule(rule_mse).await?;

    let registry_daemon_stream = registry.event_stream();

    tokio::pin!(registry_daemon_stream);
    while let Some(ev) = registry_daemon_stream.next().await {
        match ev {
            Ok(ev) => println!("My Precious: {}", ev.event_string()),
            Err(e) => println!("Error on stream -- {:#?}", e),
        };
    }

    Ok(())
}
