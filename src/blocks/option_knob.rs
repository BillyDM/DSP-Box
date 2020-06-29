use crate::{blocks, style, GuiToAudioMsg, Message, Param};

use iced::{Align, Column, Container, Element, Length, Text};

use iced_audio::{knob, IntRange, Knob};

use std::sync::mpsc;

pub struct OptionKnobBlock {
    pub label: String,
    pub int_range: IntRange,
    knob_state: knob::State<u32>,
    value_text: String,
    value: i32,
    options: Vec<String>,
}

impl OptionKnobBlock {
    pub fn new(id: u32, value: i32, default_value: i32, label: &str, options: Vec<String>) -> Self {
        let num_options = options.len() as i32;

        let int_range = IntRange::new(0, num_options - 1);

        let param = int_range.create_param(id, value, default_value);

        let mut new_knob = Self {
            label: String::from(label),
            int_range,
            knob_state: knob::State::new(param),
            value_text: String::new(),
            value,
            options,
        };

        new_knob.update_text();

        new_knob
    }

    pub fn add_params(&self, params: &mut Vec<Param>) {
        params.push(Param {
            id: self.knob_state.param.id,
            value: self.value as f32,
        });
    }

    pub fn update(&mut self, message: Message, gui_to_audio_tx: &mpsc::Sender<GuiToAudioMsg>) {
        match message {
            Message::ParamMoved(id) => {
                if self.knob_state.param.id == id {
                    self.int_range
                        .snap_normal(&mut self.knob_state.param.normal);
                    self.value = self.int_range.to_value(self.knob_state.param.normal);

                    self.update_text();

                    gui_to_audio_tx
                        .send(GuiToAudioMsg::ParamChanged(Param {
                            id: self.knob_state.param.id,
                            value: self.value as f32,
                        }))
                        .unwrap();
                }
            }
            _ => {}
        }
    }

    pub fn view(&mut self, style: &style::Theme) -> Element<Message> {
        let knob = Knob::new(&mut self.knob_state, Message::ParamMoved)
            .size(Length::from(27))
            .style(style.knob());

        let column = Column::new()
            .width(Length::from(blocks::BLOCK_WIDTH))
            .height(Length::from(blocks::BLOCK_HEIGHT))
            .align_items(Align::Center)
            .padding(10)
            .spacing(5)
            .push(knob)
            .push(Text::new(&self.label).size(12))
            .push(Text::new(&self.value_text).size(12));

        Container::new(column)
            .center_x()
            .center_y()
            .style(style.top_bar_container())
            .into()
    }

    fn update_text(&mut self) {
        self.value_text = self.options[self.value as usize].clone();
    }

    pub fn _id(&self) -> u32 {
        self.knob_state.param.id
    }

    pub fn _value(&self) -> i32 {
        self.value
    }

    pub fn _set_value(&mut self, value: i32) {
        if self.value != value {
            self.value = value;
            self.knob_state.param.normal = self.int_range.to_normal(value);

            self.update_text();
        }
    }
}
