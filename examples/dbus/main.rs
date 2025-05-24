#[macro_use]
extern crate log;

mod ui;

use adw::prelude::*;
use gtk::gdk::Display;
use ngn::phy::P2PSession;
use std::sync::{Arc, OnceLock};

pub fn rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            // 10 MiB should be plenty even for debug builds.
            .thread_stack_size(10 * 1024 * 1024)
            .enable_all()
            .build()
            .unwrap()
    })
}

async fn create_p2p_session(
    listener: Arc<ui::Listener>,
    interface_name: Option<&str>,
    device_name: &str,
) -> ngn::phy::GenericResult<Arc<ngn::phy::dbus::Session>> {
    ngn::phy::dbus::Session::new(
        ngn::phy::dbus::SessionInit {
            interface_name,
            device_name,
            go_intent: 14,
        },
        listener,
    )
    .await
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    if let Some(display) = Display::default() {
        // Add the provider to the default screen
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn main() -> gtk::glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let interface_name = std::env::var("INTERFACE_NAME").ok();
    let mut device_name = "RustTest".to_owned();
    if let Some(ref interface_name) = interface_name {
        device_name.push_str(" (");
        device_name.push_str(&interface_name);
        device_name.push_str(")");
    }

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let listener = Arc::new(ui::Listener::new(tx));
    let rt = rt();
    let session = rt.block_on(async {
        create_p2p_session(
            Arc::clone(&listener),
            interface_name.as_deref(),
            &device_name,
        )
        .await
        .unwrap()
    });

    let app = adw::Application::builder()
        .application_id(ui::APP_ID)
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_startup(|_| load_css());

    let session_clone = Arc::clone(&session);

    // HACK: We're a non-unique application, so we don't expect multiple
    // activate signals, and we can just take the receiver once. If we were to
    // use a "unique" application, we should probably create a P2P session per
    // activate signal, or something along those lines?
    let rx = std::cell::RefCell::new(Some(rx));
    app.connect_activate(move |app| {
        ui::build(
            app,
            &device_name,
            &session_clone,
            Arc::clone(&listener),
            rx.borrow_mut().take().unwrap(),
        )
    });

    rt.spawn(async move { session.wait().await });

    app.run()
}
