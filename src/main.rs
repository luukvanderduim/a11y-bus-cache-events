use atspi::cache::CacheItem;
use atspi::events::{Accessible, CacheEvent, Event, EventInterfaces, GenericEvent};

use atspi::identify::{ButtonEvent, DocumentEvents, MouseEvents, WindowEvents};
use atspi::zbus::{fdo::DBusProxy, MatchRule, MessageType};
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn Error>> {
    // set a11y on session bus
    atspi::set_session_accessibility(true).await?;
    // Open connection

    let atspi = atspi::Connection::open().await?;

    // atspi.register_event("Cache:Add").await?;
    // atspi.register_event("Cache:Remove").await?;
    atspi.register_event("Mouse").await?;
    atspi.register_event("Document").await?;
    atspi.register_event("Window").await?;

    let mouse_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Mouse")?
        .build();

    let _cache_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Cache")?
        .build();

    let document_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Document")?
        .build();

    let window_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Window")?
        .build();

    let dbus_proxy = DBusProxy::new(atspi.connection()).await?;
    //dbus_proxy.add_match_rule(mouse_rule).await?;
    // dbus_proxy.add_match_rule(cache_rule).await?;
    dbus_proxy.add_match_rule(document_rule).await?;
    dbus_proxy.add_match_rule(window_rule).await?;

    let event_stream = atspi.event_stream();

    tokio::pin!(event_stream);
    while let ev = event_stream.next().await.expect("None") {
        match ev? {
            Event::Interfaces(EventInterfaces::Mouse(mse)) => {
                println!("Mouse event ");
                //    println!("mse    {mse:#?}");
            }
            Event::Interfaces(EventInterfaces::Window(win)) => {
                println!("win event ");
                println!("mse    {win:?}");
            }
            Event::Interfaces(EventInterfaces::Object(obj)) => {
                println!("win event ");
                println!("object    {obj:?}");
            }
            _ => println!("other event "),
        };
    }

    Ok(())
}
