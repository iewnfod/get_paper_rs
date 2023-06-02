use fltk::{prelude::*, dialog};
use ui::{Message, network::*};
mod ui;

// jemalloc 优化
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// main
#[tokio::main]
async fn main() {
    // 初始化
    unsafe { ui::data::SAVE_DIR = Some("PastPapers".to_string()) };

    let mut app = fltk::app::App::default();
    app.set_scheme(fltk::app::Scheme::Gtk);
    let (sender, receiver) = fltk::app::channel::<ui::Message>();
    let mut root = fltk::window::Window::default()
        .with_size(850, 950)
        .with_label("Get CAIE Paper")
        .center_screen();
    root.resizable(&root);

    let mut buffer= ui::add_widgets(&mut root, sender);

    root.show();

    // app.run().unwrap();

    while app.wait() {
        app.redraw();
        // 刷新状态栏
        buffer.status_bar.set_value(unsafe { &ui::STATUS_BAR_CONTENT });

        // 刷新文件系统
        buffer.refresh_file_system();

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
                },
                Message::Open => {
                    // println!("Open");
                    if let Some(items) = buffer.file_system.get_selected_items() {
                        for item in items {
                            open::that(buffer.file_system.item_pathname(&item).unwrap()).unwrap();
                        }
                    }
                },
                Message::ChangeSavePath => {
                    let mut dir_dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                    dir_dialog.show();
                    let dir_path = dir_dialog.filename();
                    if dir_path.exists() {
                        if dir_path.is_dir() {
                            // 如果他是一个目录，那就修改
                            unsafe {
                                ui::data::SAVE_DIR = Some(dir_path.to_str().unwrap().to_string());
                            };
                            buffer.save_path_bt.set_label(format!("Save Path: {}", ui::data::get_save_dir()).as_str());
                            buffer.refresh_file_system();
                            buffer.close_all_nodes();
                            ui::change_status_bar_content(&format!("Change save path to {:?} successfully.", &dir_path));
                        } else if dir_path.is_file() {
                            ui::change_status_bar_content(&format!("{:?} is not a directory.", &dir_path));
                        }
                    }
                }

            }
        }
    }

}
