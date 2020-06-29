extern crate dsp_box;

use dsp_box::{Knob, Range, ZeroDBPos};

static ONE_OVER_20: f32 = 1.0 / 20.0;
#[inline]
fn db_to_amp(db: f32) -> f32 {
    10.0f32.powf(db * ONE_OVER_20)
}

struct Waveshaper {
    prev_in_gain_db: f32,
    prev_out_gain_db: f32,
    in_gain_amp: f32,
    out_gain_amp: f32,
}

impl Waveshaper {
    pub fn new() -> Self {
        Self {
            prev_in_gain_db: 0.0,
            prev_out_gain_db: 0.0,
            in_gain_amp: 1.0,
            out_gain_amp: 1.0,
        }
    }

    /// algorithm by Jon Watte from
    /// https://www.musicdsp.org/en/latest/Effects/114-waveshaper-simple-description.html
    #[inline]
    fn waveshape_distort(smp: f32) -> f32 {
        1.5f32 * smp - 0.5f32 * smp * smp * smp
    }
}

impl dsp_box::DSPBoxApp for Waveshaper {
    fn host_reset(&mut self, _sample_rate: f32) {}

    fn process_stereo(&mut self, proc_info: &mut dsp_box::AudioProcessInfo) {
        proc_info.out_l.copy_from_slice(proc_info.in_l);
        proc_info.out_r.copy_from_slice(proc_info.in_r);

        if self.prev_in_gain_db != proc_info.in_params[0] {
            self.in_gain_amp = db_to_amp(proc_info.in_params[0]);
        }
        if self.prev_out_gain_db != proc_info.in_params[1] {
            self.out_gain_amp = db_to_amp(proc_info.in_params[1]);
        }

        for smp in proc_info.out_l.iter_mut() {
            let tmp_smp = *smp * self.in_gain_amp;

            *smp = Self::waveshape_distort(tmp_smp) * self.out_gain_amp;
        }
        for smp in proc_info.out_r.iter_mut() {
            let tmp_smp = *smp * self.in_gain_amp;

            *smp = Self::waveshape_distort(tmp_smp) * self.out_gain_amp;
        }
    }
}

pub fn main() {
    let mut gui_setup = dsp_box::GuiSetup::new("Waveshaper - DSP Box");

    gui_setup.load_audio_file("audio_files/sine_c4.wav", 1.2);

    gui_setup.push_knob(Knob {
        label: "Input Gain",
        value: 0.0,
        default_value: 0.0,
        range: Range::db(-24.0, 24.0, ZeroDBPos::Center),
    });

    gui_setup.push_knob(Knob {
        label: "Output Gain",
        value: -6.0,
        default_value: 0.0,
        range: Range::db(-24.0, 24.0, ZeroDBPos::Center),
    });

    dsp_box::run(Box::new(Waveshaper::new()), gui_setup);
}
