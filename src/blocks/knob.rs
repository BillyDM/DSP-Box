use crate::{blocks, style, GuiToAudioMsg, Message, Param, Range};

use iced::{Align, Column, Container, Element, Length, Text};

use iced_audio::{knob, Knob};

use std::sync::mpsc;

pub struct KnobBlock {
    pub label: String,
    pub range: Range,
    knob_state: knob::State<u32>,
    value_text: String,
    value: f32,
}

impl KnobBlock {
    pub fn new(id: u32, value: f32, default_value: f32, label: &str, range: Range) -> Self {
        let param = blocks::create_param(id, value, default_value, &range);

        let mut new_knob = Self {
            label: String::from(label),
            range,
            knob_state: knob::State::new(param),
            value_text: String::new(),
            value: 0.0,
        };

        new_knob.update_text();

        new_knob
    }

    pub fn add_params(&self, params: &mut Vec<Param>) {
        params.push(Param {
            id: self.knob_state.param.id,
            value: self.value,
        });
    }

    pub fn update(&mut self, message: Message, gui_to_audio_tx: &mpsc::Sender<GuiToAudioMsg>) {
        match message {
            Message::ParamMoved(id) => {
                if self.knob_state.param.id == id {
                    self.update_text();

                    gui_to_audio_tx
                        .send(GuiToAudioMsg::ParamChanged(Param {
                            id: self.knob_state.param.id,
                            value: self.value,
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
        let (text, value) = blocks::normal_to_text(&mut self.knob_state.param.normal, &self.range);

        self.value_text = text;
        self.value = value;
    }

    pub fn _id(&self) -> u32 {
        self.knob_state.param.id
    }

    pub fn _value(&self) -> f32 {
        self.value
    }

    pub fn _set_value(&mut self, value: f32) {
        if self.value != value {
            self.knob_state.param.normal = match self.range {
                Range::Float(float_range) => float_range.to_normal(value),
                Range::Int(int_range) => int_range.to_normal(value as i32),
                Range::DB(db_range) => db_range.to_normal(value),
                Range::Freq(freq_range) => freq_range.to_normal(value),
            };

            self.update_text();
        }
    }
}
