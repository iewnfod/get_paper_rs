use std::{str::FromStr, time, path::Path};

use fltk::{prelude::*, *, enums::{Color, Event, Shortcut}};

pub mod data;
pub mod network;

// 状态栏内容
pub static mut STATUS_BAR_CONTENT: String = std::string::String::new();

pub fn change_status_bar_content(s: &String) {
    let mut value = String::new();
    value.push_str(" ");
    value.push_str(s);
    unsafe { STATUS_BAR_CONTENT = value };
    println!("{}", s);
}

// 保存双击时第一次点击的路径，方便和第二次比较
static mut SELECT_ITEM_PATH: String = std::string::String::new();
static mut DOUBLE_CLICK_TIMER_VEC: Vec<time::Instant> = vec![];

fn double_click_status_clear() {
    unsafe { DOUBLE_CLICK_TIMER_VEC.clear() };
    unsafe { SELECT_ITEM_PATH.clear() };
    // println!("Lose Item.");
}

#[derive(Copy, Clone)]
pub enum Message {
    Start,
    Stop,
    Open,
    ChangeSavePath,
}

#[derive(Clone)]
pub struct Buffer {
    pub check_bts: Vec<(button::CheckButton, String, String)>,  // [bt, code, label]
    pub min_year_input: input::IntInput,
    pub max_year_input: input::IntInput,
    pub status_bar: output::Output,
    pub file_system: tree::Tree,
    pub save_path_output: output::Output,
    sender: app::Sender<Message>
}

impl Buffer {
    pub fn new(sender: app::Sender<Message>) -> Buffer {
        Buffer {
            check_bts: vec![],
            min_year_input: input::IntInput::default(),
            max_year_input: input::IntInput::default(),
            status_bar: output::Output::default(),
            file_system: tree::Tree::default(),
            save_path_output: output::Output::default(),
            sender: sender
        }
    }


    pub fn refresh_file_system(&mut self) -> Result<(), hotwatch::Hotwatch> {
        // self.file_system.clear();
        // 如果不存在，那就创建
        let save_path = data::get_save_dir();
        if !std::path::PathBuf::from_str( &save_path.as_str() ).unwrap().exists() {
            std::fs::create_dir( &save_path ).unwrap();
        }

        // 判断目录级别
        let index = Path::new(&save_path).parent().unwrap().components().count();

        for f_result in walkdir::WalkDir::new(&save_path) {
            let f = match f_result {
                Ok(k) => k,
                Err(m) => {
                    return Err(self.file_system_to_default(m.to_string()));
                }
            };
            if f.file_name() == ".DS_Store" {
                continue;
            }
            let p = f.path().components();
            let p_len = p.clone().count();
            let mut final_path = String::new();
            for (i, s) in p.enumerate() {
                if i >= index {
                    final_path.push_str(s.as_os_str().to_str().unwrap());
                    if i != p_len - 1 {
                        // 如果不是最后一个，那就加上一个斜杠
                        final_path.push('/');
                    }
                }
            }
            self.file_system.add(&final_path);
        }

        // 检查，去除掉已经被删除的目录或文件
        let items = self.file_system.get_items().unwrap();
        let save_path_components = Path::new(&save_path).parent().unwrap().components();
        let mut check_path = Path::new("").to_path_buf();
        for i in save_path_components {
            check_path = check_path.join(i);
        }
        let mut need_to_remove = vec![];
        for item in items {
            let i = self.file_system.item_pathname(&item);
            let p = match i {
                Ok(k) => k,
                Err(e) => {
                    // 如果出错，那就恢复默认，并退出
                    return Err(self.file_system_to_default(e.to_string()));
                }
            };
            let path = check_path.join(Path::new(&p));
            if !path.exists() && !p.is_empty() {
                // println!("Remove: {}", &p);
                need_to_remove.push(item);
            }
        }
        for remove_item in need_to_remove {
            // 首先判断这个节点还存不存在了，然后再进行之后的操作
            if self.file_system.get_items().unwrap().contains(&remove_item) {
                if remove_item.is_root() {
                    continue;
                }
                self.file_system.remove(&remove_item).unwrap();
            }
        }
        return Ok(());
    }

    pub fn close_all_nodes(&mut self) {
        if let Some(nodes) = self.file_system.get_items() {
            let save_path = data::get_save_dir();
            let check_name = Path::new(&save_path).file_name().unwrap().to_str().unwrap();
            for mut node in nodes {
                if node.is_root() || node.label().unwrap() == check_name {
                    continue;
                }
                node.close();
            }
        }
    }

    pub fn file_system_to_default(&mut self, message: String) -> hotwatch::Hotwatch {
        change_status_bar_content(&format!("Error: `{}` occurs when changing save path.", &message));
        self.file_system.clear();
        // 重新加载保存路径，也就是重新初始化
        let watcher = super::init();

        self.save_path_output.set_value(&format!("Save Path: {}", data::get_save_dir()));
        // 刷新文件树
        return watcher;
    }

}

pub fn add_widgets(root: &mut window::Window, sender: app::Sender<Message>) -> Buffer {
    let mut buffer = Buffer::new(sender);
    // 窗口初始化
    root.set_color(Color::White);

    // 菜单栏初始化
    let mut menubar = menu::SysMenuBar::default();
    menubar.add_emit(
        "File/Open\t",
        Shortcut::Command.union(Shortcut::from_char('o')),
        menu::MenuFlag::Normal,
        sender,
        Message::Open
    );
    menubar.add_emit(
        "Operation/Start\t",
        Shortcut::Command.union(Shortcut::from_char('s')),
        menu::MenuFlag::Normal,
        sender,
        Message::Start
    );
    menubar.add_emit(
        "Operation/Stop\t",
        Shortcut::Command.union(Shortcut::Shift).union(Shortcut::from_char('s')),
        menu::MenuFlag::Normal,
        sender,
        Message::Stop
    );


    // 组件初始化
    let flex = group::Flex::default()
        .with_pos(5, 5)
        .with_size(root.width() - 10, root.height() - 70)
        .row();

        let left_flex = group::Flex::default()
            .column();

            buffer.file_system = tree::Tree::default();
            buffer.file_system.set_select_mode(tree::TreeSelect::Multi);
            buffer.file_system.set_connector_style(tree::TreeConnectorStyle::Solid);
            buffer.file_system.set_connector_color(enums::Color::Red.inactive());
            buffer.file_system.set_show_root(false);
            buffer.file_system.set_callback_reason(tree::TreeReason::Selected);
            buffer.file_system.set_sort_order(tree::TreeSort::Ascending);
            // 手动模拟双击
            // 选中以及再次按下
            buffer.file_system.handle(|t, event| {
                match event {
                    Event::Released => {
                        if let Some(items) = t.get_selected_items() {
                            if items.is_empty() {
                                // 如果是空的，那就直接清除
                                double_click_status_clear();
                            }
                            for item in items {
                                let p = t.item_pathname(&item).unwrap();
                                if unsafe { SELECT_ITEM_PATH.eq(&p) } {
                                    // 判断计时器是否超过时间限制
                                    let duration = unsafe { DOUBLE_CLICK_TIMER_VEC.last() }.unwrap().elapsed();
                                    let interval_duration = time::Duration::from_secs_f32(data::DOUBLE_CLICK_INTERVAL);
                                    // 如果间隔时间小于等于 interval，才能打开
                                    if duration <= interval_duration {
                                        // println!("Open Item: {}", p);
                                        // 转化路径，在前面添加基础路径
                                        let save_path = data::get_save_dir();
                                        let path = Path::new(&save_path);
                                        let mut final_p = path.parent().unwrap().to_path_buf();
                                        for sec in p.split('/') {
                                            final_p = final_p.join(sec);
                                        }
                                        open::that(final_p).unwrap();
                                    }
                                    double_click_status_clear();
                                } else {
                                    // println!("Select Item: {}", p);
                                    unsafe { SELECT_ITEM_PATH = p };
                                    // 添加一个计时器
                                    unsafe { DOUBLE_CLICK_TIMER_VEC.push(time::Instant::now()) };
                                }
                            }
                        }
                        true
                    },
                    Event::Unfocus | Event::Deactivate => {
                        double_click_status_clear();
                        true
                    }
                    _ => false
                }
            });

        left_flex.end();


        let right_flex = group::Flex::default()
            .column();

            for i in data::KINDS {
                let choose_bt = button::CheckButton::default()
                    .with_label(i);

                buffer.check_bts.push((choose_bt, {
                    let t: Vec<&str> = i.split(" - ").collect();
                    t[0].to_string()
                }, i.to_string()));
            }

            let year_flex = group::Flex::default()
                .row();

                buffer.min_year_input = input::IntInput::default();
                buffer.min_year_input.set_value("2022");

                let mut mid_label = output::Output::default();
                mid_label.set_value(" to");
                mid_label.set_frame(enums::FrameType::FlatBox);

                buffer.max_year_input = input::IntInput::default();
                buffer.max_year_input.set_value("2022");

            year_flex.end();

            let bts_flex = group::Flex::default()
                .row();

                let mut start_bt = button::Button::default()
                    .with_label("Start");
                start_bt.set_color(Color::White);
                start_bt.emit(buffer.sender, Message::Start);

                let mut stop_bt = button::Button::default()
                    .with_label("Stop");
                stop_bt.set_color(Color::White);
                stop_bt.emit(buffer.sender, Message::Stop);

            bts_flex.end();

        right_flex.end();

    flex.end();


    let save_path_group = group::Row::default()
        .with_size(root.width() - 10, 25)
        .with_pos(5, flex.height() + 10);

    // 修改保存路径的按钮
        buffer.save_path_output = output::Output::default();
        buffer.save_path_output.set_value( format!("Save Path: {}", data::get_save_dir()).as_str() );
        let mut save_path_bt = button::Button::default()
            .with_label( "Change Save Path" )
            .with_size(10, 25);
        save_path_bt.emit(buffer.sender, Message::ChangeSavePath);
        save_path_bt.set_color(Color::White);

    save_path_group.end();

    buffer.status_bar = output::Output::default()
        .with_size(root.width() - 10, 25)
        .with_pos(5, flex.height() + 40);

    root.end();

    buffer.refresh_file_system().unwrap();
    buffer.close_all_nodes();

    buffer
}

// 文件内容是否被修改
pub static mut IF_SAVE_DIR_CONTENT_CHANGE: bool = false;
pub static mut IF_SAVE_DIR_CHANGE: bool = false;
// 修改保存路径
pub fn change_save_path(watcher: &mut hotwatch::Hotwatch, path: &str) {
    let last_path = data::get_save_dir();
    // 修改路径
    unsafe {
        data::SAVE_DIR = Some(path.to_string());
    }
    // 修改监听路径
    match watcher.unwatch(last_path) {
        Ok(_) => (),
        Err(_) => ()
    };
    let p = data::get_save_dir();
    if !Path::new(&p).exists() {
        // 创建路径
        std::fs::create_dir_all(&p).unwrap();
    }
    println!("Start to watch: {}", &p);
    watcher.watch(&p, |e: hotwatch::Event| {
        println!("{:?}", e);
        unsafe {
            IF_SAVE_DIR_CONTENT_CHANGE = true;
        };
    }).unwrap();

    unsafe {
        IF_SAVE_DIR_CHANGE = true;
        IF_SAVE_DIR_CONTENT_CHANGE = true;
    };
}
