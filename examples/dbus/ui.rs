use adw::prelude::*;

pub const APP_ID: &str = "es.usal.P2pTest";

pub fn build(app: &adw::Application, device_name: &str) {
    let content = gtk::Box::new(gtk::Orientation::Vertical, /* spacing = */ 0);
    content.append(&adw::HeaderBar::new());

    // Create a window and set the title
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(device_name)
        .content(&content)
        .build();

    // Present window
    window.present();
}
