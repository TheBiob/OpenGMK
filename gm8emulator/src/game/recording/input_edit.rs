use crate::{
    imgui,
    input::Button,
    game::{
        replay::Input,
        recording::{
            KeyState,
            window::{
                Window,
                Openable,
                DisplayInformation
            },
        },
    },
};
use std::convert::TryInto;

pub struct InputEditWindow {
    is_open: bool,
    keys: Vec<u8>,
    states: Vec<Vec<KeyState>>,
    last_frame: usize,
}

const INPUT_TABLE_WIDTH: f32 = 50.0;
const INPUT_TABLE_HEIGHT: f32 = 20.0;

impl Window for InputEditWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        // todo: figure out a better system on when to update this.
        if self.last_frame != info.config.current_frame {
            self.last_frame = info.config.current_frame;
            self.update_keys(info);
        }
        unsafe { cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_WindowPadding.try_into().unwrap(), imgui::Vec2(0.0, 0.0).into()); }
        info.frame.begin_window(Self::window_name(), None, true, false, Some(&mut self.is_open));

        if info.frame.begin_table(
            "Input",
            self.keys.len() as i32 + 1,
            (cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_RowBg
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_Borders
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadInnerX) as i32,
            imgui::Vec2(0.0, 0.0),
            0.0
        ) {
            info.frame.table_setup_column("Frame", 0, 0.0);
            for key in self.keys.iter() {
                if let Some(button) = Button::try_from_u8(*key) {
                    info.frame.table_setup_column(&format!("{}", button), cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as i32, INPUT_TABLE_WIDTH);
                }
            }

            info.frame.table_headers_row();
            self.draw_input_rows(info);
            info.frame.end_table();
        }

        unsafe { cimgui_sys::igPopStyleVar(1); }
        info.frame.end();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn name(&self) -> String {
        Self::window_name().to_owned()
    }
}
impl Openable<Self> for InputEditWindow {
    fn window_name() -> &'static str {
        "Input Editor"
    }

    fn open() -> Self {
        Self::new()
    }
}

impl InputEditWindow {
    fn new() -> Self {
        Self {
            is_open: true,
            keys: Vec::new(),
            states: Vec::new(),
            // keys: vec![0x26, 0x28, 0x27, 0x25],
            // up, down, left, right
            last_frame: 0,
        }
    }

    fn update_keys(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            replay,
            ..
        } = info;

        self.keys.clear();
        for i in 0..replay.frame_count() {
            if let Some(frame) = replay.get_frame(i) {
                for input in &frame.inputs {
                    match input {
                        Input::KeyPress(btn) | Input::KeyRelease(btn) => {
                            if !self.keys.contains(btn) {
                                self.keys.push(*btn);
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
        self.states.clear();
        self.states.reserve(replay.frame_count());
        self.states.push(vec![KeyState::Neutral; self.keys.len()]);
        
        for i in 0..replay.frame_count() {
            if let Some(frame) = replay.get_frame(i) {
                for input in &frame.inputs {
                    match input {
                        Input::KeyPress(current_key) | Input::KeyRelease(current_key)  => {
                            if let Some(index) = self.keys.iter().position(|k| k == current_key) {
                                self.update_keystate(i, index, matches!(input, Input::KeyPress(_)));
                            }
                        },
                        _ => {},
                    }
                }

                self.states.push(self.states[i].clone());
                for state in self.states[i + 1].iter_mut() {
                    *state = match state {
                        KeyState::NeutralWillPress
                            | KeyState::NeutralWillTriple
                            | KeyState::HeldWillDouble
                            | KeyState::HeldDoubleEveryFrame
                            | KeyState::Held
                            => KeyState::Held,
                        KeyState::NeutralWillDouble
                            | KeyState::NeutralDoubleEveryFrame
                            | KeyState::HeldWillRelease
                            | KeyState::NeutralWillCactus
                            | KeyState::HeldWillTriple
                            | KeyState::Neutral
                            => KeyState::Neutral
                    }
                }
            }
        }
    }

    fn draw_input_rows(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            replay,
            frame,
            ..
        } = info;

        for i in 0..replay.frame_count() {
            frame.table_next_row(0, INPUT_TABLE_HEIGHT);

            frame.table_set_column_index(0);
            frame.text(&format!("{}", i + 1));

            for j in 0..self.keys.len() {
                frame.table_set_column_index(j as i32 + 1);
                let keystate = &self.states[i][j];
                frame.invisible_button(keystate.repr(), imgui::Vec2(INPUT_TABLE_WIDTH, INPUT_TABLE_HEIGHT), None);
                keystate.draw_keystate(frame, frame.get_item_rect_min()-frame.window_position(), frame.get_item_rect_size());
            }
        }
    }

    fn update_keystate(&mut self, frame_index: usize, key_index: usize, pressed: bool) {
        let state = &mut self.states[frame_index][key_index];

        macro_rules! invalid {
            () => {{
                    println!("Warning: Invalid input order {}: {}", state.repr(), pressed);

                KeyState::Neutral
            }};
        }
        
        *state = match state {
                    KeyState::Held => if pressed { /* this one is not currently possible to enter in tas mode */ KeyState::Held } else { KeyState::HeldWillRelease },
                    KeyState::HeldWillRelease => if pressed { KeyState::HeldWillDouble } else { invalid!() },
                    KeyState::HeldWillDouble => if pressed { invalid!() } else { KeyState::HeldWillTriple },
                    KeyState::HeldWillTriple => invalid!(),

                    KeyState::Neutral => if pressed { KeyState::NeutralWillPress } else { KeyState::NeutralWillCactus },
                    KeyState::NeutralWillPress => if pressed { invalid!() } else { KeyState::NeutralWillDouble },
                    KeyState::NeutralWillDouble => if pressed { KeyState::NeutralWillTriple } else { invalid!() },
                    KeyState::NeutralWillTriple => invalid!(),

                    _ => if pressed { KeyState::NeutralWillPress } else { KeyState::HeldWillRelease },
                };
    }
}
