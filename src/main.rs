use atspi::cache::CacheItem;
use atspi::events::{CacheEvent, Event};
use atspi::StateSet;

use atspi::identify::ButtonEvent;
use atspi::zbus::{fdo::DBusProxy, MatchRule, MessageType};
use std::error::Error;
use std::ffi::c_short;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    // set a11y on session bus
    atspi::set_session_accessibility(true).await?;
    // Open connection
    let atspi = atspi::Connection::open().await?;

    atspi.register_event("Cache:Add").await?;
    atspi.register_event("Cache:Remove").await?;
    atspi.register_event("Mouse:Button").await?;

    let mouse_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Mouse")?
        .build();

    let cache_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Cache")?
        .build();

    let dbus_proxy = DBusProxy::new(atspi.connection()).await?;
    dbus_proxy.add_match_rule(mouse_rule).await?;
    let dbus_proxy = DBusProxy::new(atspi.connection()).await?;
    dbus_proxy.add_match_rule(cache_rule).await?;

    let event_stream = atspi.event_stream();

    tokio::pin!(event_stream);
    while let Some(ev) = event_stream.next().await {
        match ev {
            Ok(ev) => match ev {
                Event::Atspi(aev) => {
                    let button_ev = ButtonEvent::try_from(aev)?;
                    println!(
                        "The {} button press at: {},{}",
                        button_ev.button(),
                        button_ev.x(),
                        button_ev.y()
                    );
                }
                Event::Cache(cev) => match cev {
                    CacheEvent::Add(caev) => {
                        let item: CacheItem = caev.body;
                        let CacheItem {
                            object,
                            app,
                            parent,
                            index,
                            children,
                            ifaces,
                            short_name,
                            role,
                            name,
                            states,
                        } = item;

                        println!(
                            "Cache Add:    
                        object: {object:?}
                        app: {app:?}
                        parent: {parent:?}
                        index in parent {index:?}
                        number of children: {children:?}
                        interfaceset: {ifaces:#?}
                        short name: {short_name},
                        role: {role:?}
                        name: {name}
                        states: {states:#?}  "
                        );
                    }
                    CacheEvent::Remove(crev) => println!("Cache remove: {crev:?}"),
                },
            },

            Err(e) => println!("Error on stream -- {e:#?}"),
        };
    }

    Ok(())
}
