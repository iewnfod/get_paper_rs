use fltk::{prelude::*};
use ui::{Message, network::*};
mod ui;

// main
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
    let mut root = fltk::window::Window::default()
        .with_size(850, 950)
        .with_label("Get CAIE Paper")
        .center_screen();
    root.resizable(&root);

    let (mut buffer, mut file_system) = ui::add_widgets(&mut root, sender);

    root.show();

    // app.run().unwrap();

    while app.wait() {
        app.redraw();
        // 刷新状态栏
        buffer.status_bar.set_value(unsafe { &ui::STATUS_BAR_CONTENT });

        // 刷新文件系统
        ui::refresh_file_system(&mut file_system, ui::data::SAVE_DIR);

        if let Some(msg) = receiver.recv() {
            match msg {
                Message::Start => {
                    if unsafe { !DOWNLOADING } {
                        println!("Start");

                        unsafe { DOWNLOADING = true };

                        // 对数据进行预处理
                        let min_year: isize = buffer.min_year_input.value().parse().unwrap();
                        let max_year: isize = buffer.max_year_input.value().parse().unwrap();
                        let check_bts_vec = buffer.check_bts.clone();
                        let mut check_bts = vec![];
                        for (bt, code, name) in check_bts_vec.iter() {
                            check_bts.push((bt.value(), code.clone(), name.clone()));
                        }

                        ui::change_status_bar_content(&"Start!".to_string());

                        tokio::spawn( async move {
                            println!("Spawn Start");

                            start(min_year, max_year, check_bts).await;
                            unsafe { DOWNLOADING = false };
                            println!("Spawn Finish");
                            // return ;
                        });
                    } else {
                        println!("Last download have not finished. Please try again after it is finished. ");
                    }
                },
                Message::Stop => {
                    if unsafe { DOWNLOADING } {
                        println!("Trying to stop. ");
                        ui::change_status_bar_content(&"Trying to stop".to_string());
                        unsafe { DOWNLOADING = false };
                    }
                }
            }
        }
    }

}
