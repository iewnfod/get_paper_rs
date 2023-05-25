use fltk::{prelude::*, *, enums::{Color, Event}};

pub mod data;
pub mod network;

// 状态栏内容
pub static mut STATUS_BAR_CONTENT: String = std::string::String::new();

pub fn change_status_bar_content(s: &String) {
    unsafe { STATUS_BAR_CONTENT = s.to_string() };
}

// 保存双击时第一次点击的路径，方便和第二次比较
static mut SELECT_ITEM_PATH: String = std::string::String::new();

#[derive(Copy, Clone)]
pub enum Message {
    Start
}

#[derive(Clone)]
pub struct Buffer {
    pub check_bts: Vec<(button::CheckButton, String, String)>,  // [bt, code, label]
    pub min_year_input: input::IntInput,
    pub max_year_input: input::IntInput,
    pub status_bar: output::Output,
    file_system: tree::Tree,
    sender: app::Sender<Message>
}

impl Buffer {
    pub fn new(sender: app::Sender<Message>) -> Buffer {
        Buffer {
            check_bts: vec![],
            min_year_input: input::IntInput::default(),
            max_year_input: input::IntInput::default(),
            file_system: tree::Tree::default(),
            status_bar: output::Output::default(),
            sender: sender
        }
    }

    pub fn refresh_file_system(&mut self, root_path: &str) {
        // self.file_system.clear();
        for f_result in walkdir::WalkDir::new(root_path) {
            let f = f_result.unwrap();
            if f.file_name() == ".DS_Store" {
                continue;
            }
            self.file_system.add(f.path().to_str().unwrap());
        }
    }

    pub fn close_all_nodes(&mut self) {
        let nodes = self.file_system.get_items().unwrap();
        for mut node in nodes {
            if node.is_root() || node.label().unwrap() == data::SAVE_DIR {
                continue;
            }
            node.close();
        }
    }
}

pub fn add_widgets(root: &mut window::Window, sender: app::Sender<Message>) -> Buffer {
    let mut buffer = Buffer::new(sender);
    // 窗口初始化
    root.set_color(Color::White);

    // 组件初始化
    let flex = group::Flex::default()
        .with_pos(5, 5)
        .with_size(root.width() - 10, root.height() - 40)
        .row();

        let left_flex = group::Flex::default()
            .column();

            buffer.file_system = tree::Tree::default();
            buffer.file_system.set_select_mode(tree::TreeSelect::Multi);
            buffer.file_system.set_connector_style(tree::TreeConnectorStyle::Solid);
            buffer.file_system.set_connector_color(enums::Color::Red.inactive());
            buffer.file_system.set_show_root(false);
            buffer.file_system.set_callback_reason(tree::TreeReason::Selected);
            // 手动模拟双击
            // 选中以及再次按下
            buffer.file_system.handle(|t, event| {
                match event {
                    Event::Released => {
                        if let Some(items) = t.get_selected_items() {
                            for item in items {
                                let p = t.item_pathname(&item).unwrap();
                                if unsafe { SELECT_ITEM_PATH.eq(&p) } {
                                    println!("Open Item: {}", p);
                                    open::that(p).unwrap();
                                } else {
                                    println!("Select Item: {}", p);
                                    unsafe { SELECT_ITEM_PATH = p };
                                }
                            }
                        }
                        true
                    },
                    Event::Unfocus => {
                        unsafe { SELECT_ITEM_PATH.clear() };
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
                mid_label.set_value("to");
                mid_label.set_frame(enums::FrameType::FlatBox);

                buffer.max_year_input = input::IntInput::default();
                buffer.max_year_input.set_value("2022");

            year_flex.end();

            let mut start_bt = button::Button::default()
                .with_label("Start Download");
            start_bt.set_color(Color::White);
            start_bt.emit(buffer.sender, Message::Start);


        right_flex.end();

    flex.end();

    buffer.status_bar = output::Output::default()
        .with_size(root.width() - 10, 25)
        .with_pos(5, flex.height() + 10);

    root.end();

    buffer.refresh_file_system(data::SAVE_DIR);
    buffer.close_all_nodes();
    buffer

}
