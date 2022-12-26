use atspi::cache::CacheItem;
use atspi::events::{Accessible, CacheEvent, Event, GenericEvent};

use atspi::identify::{ButtonEvent, DocumentEvents};
use atspi::zbus::{fdo::DBusProxy, MatchRule, MessageType};
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn Error>> {
    // set a11y on session bus
    atspi::set_session_accessibility(true).await?;
    // Open connection

    let atspi = atspi::Connection::open().await?;

    atspi.register_event("Cache:Add").await?;
    atspi.register_event("Cache:Remove").await?;
    atspi.register_event("Mouse:Button").await?;
    atspi.register_event("Document").await?;

    let mouse_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Mouse")?
        .member("Button")?
        .build();

    let cache_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Cache")?
        .build();

    let document_rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface("org.a11y.atspi.Event.Document")?
        .build();

    let dbus_proxy = DBusProxy::new(atspi.connection()).await?;
    dbus_proxy.add_match_rule(mouse_rule).await?;
    dbus_proxy.add_match_rule(cache_rule).await?;
    dbus_proxy.add_match_rule(document_rule).await?;

    let event_stream = atspi.event_stream();

    tokio::pin!(event_stream);
    while let Some(ev) = event_stream.next().await {
        match ev {
            Ok(ev) => match ev {
                Event::Atspi(aev) => {
                    if aev.member().unwrap().as_str() == "Document" {
                        let doc_ev = DocumentEvents::from(aev.clone());
                        {
                            println!("What happened to the doc? {doc_ev:#?}");
                        }
                    }

                    let button_ev = ButtonEvent::try_from(aev)?;
                    println!(
                        "Button: {} at: {},{}",
                        button_ev.button(),
                        button_ev.x(),
                        button_ev.y()
                    );
                }
                Event::Cache(cev) => match cev {
                    CacheEvent::Add(caev) => {
                        let item: &CacheItem = caev.item();
                        println!("CacheItem:  {item:#?}")
                    }
                    CacheEvent::Remove(crev) => {
                        let acc: &Accessible = crev.as_accessible();
                        println!("Removed Accessible: {acc:?}");
                    }
                },
                _ => println!("Unmatched event"),
            },

            Err(e) => println!("Error on stream -- {e:#?}"),
        };
    }

    Ok(())
}
