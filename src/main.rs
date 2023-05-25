use fltk::prelude::*;
use ui::Message;
mod ui;


// main
#[tokio::main(worker_threads = 3)]
async fn main() {
    // 初始化一些东西
    let save_path = std::path::Path::new(ui::data::SAVE_DIR);
    if !save_path.exists() {
        std::fs::create_dir(ui::data::SAVE_DIR).unwrap();
    }

    let mut app = fltk::app::App::default();
    app.set_scheme(fltk::app::Scheme::Gtk);
    let (sender, receiver) = fltk::app::channel::<ui::Message>();
    let mut root = fltk::window::Window::new(100, 100, 850, 930, "Main Window");
    root.resizable(&root);

    let mut buffer: ui::Buffer = ui::add_widgets(&mut root, sender);

    root.show();

    // app.run().unwrap();

    while app.wait() {
        app.redraw();
        // 刷新状态栏
        buffer.status_bar.set_value(unsafe { &ui::STATUS_BAR_CONTENT });
        // 刷新文件系统
        buffer.refresh_file_system(ui::data::SAVE_DIR);

        if let Some(msg) = receiver.recv() {
            match msg {
                Message::Start => {
                    let buffer_clone = buffer.clone();
                    println!("Start");

                    tokio::spawn( async {
                        ui::network::start(buffer_clone).await;
                    });
                }
            }
        }
    }
}
