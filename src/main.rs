use std::{str::FromStr, io::{Read}};

use fltk::{prelude::*, dialog, enums};
use ui::{Message, network::*};
mod ui;

// jemalloc 优化
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// main
#[tokio::main]
async fn main() {
    // 初始化
    init();

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

                            // 修改配置文件
                            refresh_config_content(false);

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

// init 表示是否是初始化时的调用，如果是，那就不会进行根据现有内容写入的操作
fn refresh_config_content(init: bool) {
    let config_path = format!("{}/{}", ui::data::BASE_DIR, ui::data::CONFIG_PATH);
    let config = std::path::Path::new(&config_path);
    // 如果不存在，并且需要初始化，那才需要写入默认数值
    if !config.exists() && init {
        std::fs::write(config, ui::data::DEFAULT_CONFIG_CONTENT).unwrap();
    } else if !init {
        // 否则，一律生成设置文本并写入
        std::fs::write(config, generate_config_content()).unwrap();
    }
}

fn generate_config_content() -> String {
    let content = format!(
"save_dir={}
width={}
height={}",
        ui::data::get_save_dir(),
        unsafe {ui::data::WIDTH},
        unsafe {ui::data::HEIGHT}
    );

    content
}

fn init() {
    // 尝试读取设置
    let base = std::path::PathBuf::from_str( ui::data::BASE_DIR ).unwrap();
    // 如果保存路径不存在，那就创建
    if !base.exists() {
        std::fs::create_dir_all(ui::data::BASE_DIR).unwrap();
    }

    // 设置路径
    refresh_config_content(true);

    // 加载设置
    let config_path = format!("{}/{}", ui::data::BASE_DIR, ui::data::CONFIG_PATH);
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
                unsafe { ui::data::SAVE_DIR = Some(value.to_string()) };
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
}
