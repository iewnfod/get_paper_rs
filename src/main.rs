use std::{io::{Read}, path::Path};

use fltk::{prelude::*, dialog, enums};
use ui::{Message, network::*};
mod ui;

// jemalloc 优化 (已删除，编译警告未测试，并且编译失败)
// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// main
#[tokio::main]
async fn main() {
    // 初始化
    let mut watcher = init();

    // 软件运行
    let mut app = fltk::app::App::default();
    app.set_scheme(fltk::app::Scheme::Gtk);
    let (sender, receiver) = fltk::app::channel::<ui::Message>();
    let mut root = fltk::window::Window::default()
        .with_size(unsafe {ui::data::WIDTH}, unsafe {ui::data::HEIGHT})
        .center_screen()
        .with_label("Get CAIE Paper");
    root.resizable(&root);
    root.handle({
        move |w, event| {
            match event {
                enums::Event::Resize => {
                    unsafe {
                        ui::data::WIDTH = w.width();
                        ui::data::HEIGHT = w.height();
                    };
                    refresh_config_content(false);
                    true
                }
                _ => false
            }
        }
    });

    let mut buffer= ui::add_widgets(&mut root, sender);

    root.show();

    let mut download_threads: Vec<tokio::task::JoinHandle<()>> = vec![];

    while app.wait() {
        app.redraw();
        // 刷新状态栏
        buffer.status_bar.set_value(unsafe { &ui::STATUS_BAR_CONTENT });

        // 刷新文件系统
        if unsafe { ui::IF_SAVE_DIR_CONTENT_CHANGE } {
            match buffer.refresh_file_system() {
                Ok(_) => (),
                Err(new_watcher) => {
                    watcher = new_watcher;
                }
            };
            if unsafe { ui::IF_SAVE_DIR_CHANGE } {
                buffer.close_all_nodes();
            }
            unsafe {
                ui::IF_SAVE_DIR_CONTENT_CHANGE = false;
                ui::IF_SAVE_DIR_CHANGE = false;
            };
        }

        // 查看下载是否还在进行
        if !download_threads.is_empty() && download_threads.first().unwrap().is_finished() {
            download_threads.clear();
            unsafe { DOWNLOADING = false };
        }


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
                        println!("Check buttons: {:?}", check_bts);

                        ui::change_status_bar_content(&"Start!".to_string());

                        download_threads.push(
                            tokio::spawn( async move {
                                println!("Spawn Start");

                                start(min_year, max_year, check_bts).await;
                                unsafe { DOWNLOADING = false };
                                println!("Spawn Finish");
                                // return ;
                            })
                        )
                    } else {
                        ui::change_status_bar_content(&"Last download has not finished. Please try again after it is finished. ".to_string());
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
                        let save_path = Path::new(&ui::data::get_save_dir()).parent().unwrap().to_path_buf();
                        for item in items {
                            let item_path = Path::new(&buffer.file_system.item_pathname(&item).unwrap()).to_path_buf();
                            open::that(save_path.clone().join(&item_path)).unwrap();
                        }
                    }
                },
                Message::ChangeSavePath => {
                    let mut dir_dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                    dir_dialog.show();
                    let dir_path = dir_dialog.filename();
                    if dir_path.exists() {
                        if dir_path.is_dir() {
                            ui::change_status_bar_content(&format!("Changing save path to {:?}.", &dir_path));

                            // 如果他是一个目录，那就修改
                            ui::change_save_path(&mut watcher, dir_path.to_str().unwrap());

                            buffer.save_path_output.set_value(format!(" Save Path: {}", ui::data::get_save_dir()).as_str());
                            // 清空文件树，并刷新
                            buffer.file_system.clear();
                            match buffer.refresh_file_system() {
                                Ok(_) => (),
                                Err(new_watcher) => {
                                    watcher = new_watcher;
                                }
                            };
                            buffer.close_all_nodes();
                            // 修改配置文件
                            refresh_config_content(false);
                        } else if dir_path.is_file() {
                            ui::change_status_bar_content(&format!("{:?} is not a directory.", &dir_path));
                        }
                    }
                },

                Message::ResetSavePath => {
                    let default_path = ui::data::get_default_save_dir();
                    ui::change_save_path(&mut watcher, &default_path);
                    buffer.save_path_output.set_value(format!(" Save Path: {}", &default_path).as_str());

                    // 清空文件树，并刷新
                    buffer.file_system.clear();
                    match buffer.refresh_file_system() {
                        Ok(_) => (),
                        Err(new_watcher) => {
                            watcher = new_watcher;
                        }
                    };
                    buffer.close_all_nodes();
                    // 修改配置文件
                    refresh_config_content(false);
                }

            }
        }
    }

}

// init 表示是否是初始化时的调用，如果是，那就不会进行根据现有内容写入的操作
fn refresh_config_content(init: bool) {
    let config_path = ui::data::base_dir().join(ui::data::CONFIG_PATH);
    let config = std::path::Path::new(&config_path);
    // 如果不存在，并且需要初始化，那才需要写入默认数值
    if !config.exists() && init {
        std::fs::write(config, ui::data::default_config_content()).unwrap();
    } else if !init {
        // 否则，一律生成设置文本并写入
        std::fs::write(config, generate_config_content()).unwrap();
    }
}

fn generate_config_content() -> String {
    let content = format!("
save_dir={}
width={}
height={}",
        ui::data::get_save_dir(),
        unsafe {ui::data::WIDTH},
        unsafe {ui::data::HEIGHT}
    );

    content
}

fn init() -> hotwatch::Hotwatch {
    // 创建监视
    let mut watcher = hotwatch::Hotwatch::new().unwrap();
    // 尝试读取设置
    let base = ui::data::base_dir();
    // 如果保存路径不存在，那就创建
    if !base.exists() {
        std::fs::create_dir_all(ui::data::base_dir()).unwrap();
    }

    // 设置路径
    refresh_config_content(true);

    // 加载设置
    let config_path = ui::data::base_dir().join(ui::data::CONFIG_PATH);
    let mut config_file = std::fs::File::open(config_path).unwrap();
    let mut config_content = String::new();
    config_file.read_to_string(&mut config_content).unwrap();
    let config_content: Vec<&str> = config_content.trim().split('\n').collect();
    for item in config_content.iter() {
        let item_data: Vec<&str> = item.split('=').collect();
        let key = item_data[0].trim();
        let value = item_data[1].trim();

        match key {
            "save_dir" => {
                ui::change_save_path(&mut watcher, value);
            },
            "width" => {
                unsafe { ui::data::WIDTH = value.parse().unwrap() };
            },
            "height" => {
                unsafe { ui::data::HEIGHT = value.parse().unwrap() };
            }
            _ => {}
        }
    }

    watcher
}
