use fltk::prelude::*;
use ui::Message;
mod ui;

#[tokio::main]
async fn main() {
    // 初始化一些东西
    let save_path = std::path::Path::new(ui::data::SAVE_DIR);
    if !save_path.exists() {
        std::fs::create_dir(ui::data::SAVE_DIR).unwrap();
    }

    let mut app = fltk::app::App::default();
    app.set_scheme(fltk::app::Scheme::Gtk);
    let (sender, receiver) = fltk::app::channel::<ui::Message>();
    let mut root = fltk::window::Window::new(100, 100, 850, 900, "Main Window");
    root.resizable(&root);

    let mut buffer = ui::add_widgets(&mut root, sender);

    root.show();

    // app.run().unwrap();

    while app.wait() {
        app.redraw();
        if let Some(msg) = receiver.recv() {
            match msg {
                Message::Start => {
                    let buffer_clone = buffer.clone();
                    println!("Start");
                    ui::network::start(buffer_clone).await;
                    buffer.refresh_file_system();  // 刷新左侧文件系统
                }
            }
        }
    }
}
