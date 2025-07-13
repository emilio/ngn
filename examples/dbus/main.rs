#[macro_use]
extern crate log;

mod ui;

use adw::prelude::*;
use gtk::gdk::Display;

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
    let interface_name = std::env::var("INTERFACE_NAME").ok();
    let mut device_name = "RustTest".to_owned();
    if let Some(ref interface_name) = interface_name {
        device_name.push_str(" (");
        device_name.push_str(&interface_name);
        device_name.push_str(")");
    }

    let iface_for_logging = interface_name.clone().unwrap_or_default();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .format(move |buf, record| {
            use std::io::Write;
            write!(buf, "[{}", record.level())?;
            if let Some(f) = record.file() {
                write!(buf, " {}:{}", f, record.line().unwrap_or(0))?;
            }
            if !iface_for_logging.is_empty() {
                write!(buf, " {}", iface_for_logging)?;
            }
            writeln!(buf, " p:{}] {}", std::process::id(), record.args())
        })
        .init();

    let app = adw::Application::builder()
        .application_id(ui::APP_ID)
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(move |app| {
        ui::build(
            app,
            &device_name,
            interface_name.as_deref(),
        )
    });

    app.run()
}
