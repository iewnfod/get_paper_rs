use fltk::prelude::*;
use ui::Message;
mod ui;

fn get_result_path(changed_path: std::path::PathBuf) -> String {
    let path_str = changed_path.to_str().unwrap();
    let split_path: Vec<&str> = path_str.split('/').collect();
    let mut result_path = std::string::String::from(ui::data::SAVE_DIR);
    let mut save_dir_level = split_path.len();
    for path_level in 0..split_path.len() {
        // 如果到了那一级，那就标记为这个位置
        if split_path[path_level] == ui::data::SAVE_DIR {
            save_dir_level = path_level;
        }
        // 如果在后面就表示要加进去了
        if path_level > save_dir_level {
            result_path.push_str(format!("/{}", split_path[path_level]).as_str());
        }
    }
    // println!("Result Path: {}", result_path);
    result_path
}

#[derive(Clone, Copy)]
enum FileSystemOperation {
    Create,
    Remove,
    None
}

static mut FILE_SYSTEM_REFRESH_PATH: String = std::string::String::new();
static mut FILE_SYSTEM_OPERATION: FileSystemOperation = FileSystemOperation::None;


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
    let mut root = fltk::window::Window::new(100, 100, 850, 930, "Main Window");
    root.resizable(&root);

    let mut buffer: ui::Buffer = ui::add_widgets(&mut root, sender);

    let mut watcher = hotwatch::Hotwatch::new().expect("Failed to initialize hotwatch!");
    watcher.watch(ui::data::SAVE_DIR,
        |e: hotwatch::Event| {
            println!("{:?}", e);
            if let hotwatch::Event::Create(changed_path) = e {
                // 如果是写入文件
                unsafe { FILE_SYSTEM_REFRESH_PATH = get_result_path(changed_path) };
                unsafe { FILE_SYSTEM_OPERATION = FileSystemOperation::Create };
            } else if let hotwatch::Event::Remove(changed_path) = e {
                unsafe { FILE_SYSTEM_REFRESH_PATH = get_result_path(changed_path) };
                unsafe { FILE_SYSTEM_OPERATION = FileSystemOperation::Remove };
            } else if let hotwatch::Event::Rename(original_path, target_path) = e {
                unsafe { FILE_SYSTEM_REFRESH_PATH = get_result_path(original_path) };
                unsafe { FILE_SYSTEM_OPERATION = FileSystemOperation::Remove };
                unsafe { FILE_SYSTEM_REFRESH_PATH = get_result_path(target_path) };
                unsafe { FILE_SYSTEM_OPERATION = FileSystemOperation::Create };
            }
        }
    ).unwrap();

    root.show();

    // app.run().unwrap();

    while app.wait() {
        // 刷新状态栏
        buffer.status_bar.set_value(unsafe { &ui::STATUS_BAR_CONTENT });
        // 文件系统刷新机制
        if unsafe { !FILE_SYSTEM_REFRESH_PATH.is_empty() } {
            println!("Refresh Path: {}", unsafe { FILE_SYSTEM_REFRESH_PATH.as_str() });
            let file_system_refresh_path = unsafe { FILE_SYSTEM_REFRESH_PATH.as_str() };
            match unsafe { FILE_SYSTEM_OPERATION } {
                FileSystemOperation::Create => {
                    buffer.refresh_file_system(file_system_refresh_path);
                },
                FileSystemOperation::Remove => {
                    buffer.remove_file_system_node(file_system_refresh_path);
                },
                FileSystemOperation::None => {}
            }
            unsafe { FILE_SYSTEM_REFRESH_PATH.clear() };
            unsafe { FILE_SYSTEM_OPERATION = FileSystemOperation::None };
        }

        if let Some(msg) = receiver.recv() {
            match msg {
                Message::Start => {
                    let buffer_clone = buffer.clone();
                    println!("Start");
                    tokio::spawn(async move {
                        ui::network::start(buffer_clone).await;
                    });
                }
            }
        }
    }
}
