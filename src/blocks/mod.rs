use iced_audio::{Normal, Param};

use crate::Range;

mod knob;
mod option_knob;
pub use knob::KnobBlock;
pub use option_knob::OptionKnobBlock;

static BLOCK_WIDTH: u16 = 65;
static BLOCK_HEIGHT: u16 = 95;

pub enum Block {
    Knob(KnobBlock),
    OptionKnob(OptionKnobBlock),
}

#[derive(Copy, Clone)]
pub enum DecimalPlaces {
    One,
    Two,
}

fn float_text(value: f32, decimal_places: DecimalPlaces) -> String {
    match decimal_places {
        DecimalPlaces::One => format!("{:.1}", value),
        DecimalPlaces::Two => format!("{:.2}", value),
    }
}

fn normal_to_text(normal: &mut Normal, range: &Range) -> (String, f32) {
    match range {
        Range::Float(float_range) => {
            let value = float_range.to_value(*normal);
            (float_text(value, DecimalPlaces::Two), value)
        }
        Range::Int(int_range) => {
            int_range.snap_normal(normal);
            let value = int_range.to_value(*normal);
            (format!("{}", value), value as f32)
        }
        Range::DB(db_range) => {
            let value = db_range.to_value(*normal);
            (
                format!("{} dB", float_text(value, DecimalPlaces::One)),
                value,
            )
        }
        Range::Freq(freq_range) => {
            let value = freq_range.to_value(*normal);
            if value < 1000.0 {
                (
                    format!("{} Hz", float_text(value, DecimalPlaces::One)),
                    value,
                )
            } else {
                (
                    format!("{} kHz", float_text(value / 1000.0, DecimalPlaces::Two)),
                    value,
                )
            }
        }
    }
}

pub fn create_param(id: u32, value: f32, default_value: f32, range: &Range) -> Param<u32> {
    match range {
        Range::Float(float_range) => float_range.create_param(id, value, default_value),
        Range::Int(int_range) => int_range.create_param(id, value as i32, default_value as i32),
        Range::DB(db_range) => db_range.create_param(id, value, default_value),
        Range::Freq(freq_range) => freq_range.create_param(id, value, default_value),
    }
}
