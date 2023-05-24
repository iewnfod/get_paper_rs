use fltk::{prelude::*, *, enums::Color};

pub mod data;
pub mod network;

#[derive(Copy, Clone)]
pub enum Message {
    Start
}

#[derive(Clone)]
pub struct Buffer {
    pub check_bts: Vec<(button::CheckButton, String, String)>,  // [bt, code, label]
    pub min_year_input: input::IntInput,
    pub max_year_input: input::IntInput,
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
            sender: sender
        }
    }

    pub fn refresh_file_system(&mut self) {
        self.file_system.clear();
        for f_result in walkdir::WalkDir::new(data::SAVE_DIR) {
            let f = f_result.unwrap();
            if f.file_name() == ".DS_Store" {
                continue;
            }
            self.file_system.add(f.path().to_str().unwrap());
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
        .with_size(root.width() - 10, root.height() - 10)
        .row();

        let left_flex = group::Flex::default()
            .column();

            buffer.file_system = tree::Tree::default();
            buffer.file_system.set_select_mode(tree::TreeSelect::Multi);
            buffer.file_system.set_connector_style(tree::TreeConnectorStyle::Solid);
            buffer.file_system.set_connector_color(enums::Color::Red.inactive());
            buffer.file_system.set_show_root(false);
            buffer.file_system.set_callback_reason(tree::TreeReason::Selected);
            buffer.file_system.set_callback(|t| {
                if let Some(items) = t.get_selected_items() {
                    for item in items {
                        let p = t.item_pathname(&item).unwrap();
                        println!("Open File: {}", p);
                        open::that(p).unwrap();
                    }
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
                mid_label.set_frame(enums::FrameType::NoBox);
                mid_label.set_readonly(true);

                buffer.max_year_input = input::IntInput::default();
                buffer.max_year_input.set_value("2022");

            year_flex.end();


            let mut start_bt = button::Button::default()
                .with_label("Start");
            start_bt.set_color(Color::White);
            start_bt.emit(buffer.sender, Message::Start);

        right_flex.end();

    flex.end();

    root.end();

    buffer.refresh_file_system();
    buffer

}
