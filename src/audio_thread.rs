use crate::DSPBoxApp;
use std::marker::Send;
use std::sync::mpsc;

use crate::audio_player::{AudioPlayer, AudioPlayerBuffer};

pub enum GuiToAudioMsg {
    ParamChanged(Param),
    InitPreset(Vec<Param>),
    LoadAudioPlayerBuffer(AudioPlayerBuffer),
    Play,
    Pause,
    Stop,
    Bypass,
    Unbypass,
    Panic,
}

#[derive(Copy, Clone)]
pub struct Param {
    pub id: u32,
    pub value: f32,
}

pub struct AudioProcessInfo<'a> {
    pub in_l: &'a [f32],
    pub in_r: &'a [f32],
    pub out_l: &'a mut [f32],
    pub out_r: &'a mut [f32],
    pub sample_rate: f32,
    pub in_params: &'a Vec<f32>,
}

pub struct AudioThread {
    sample_rate: f32,
    dsp_app: Box<dyn DSPBoxApp + Send>,
    gui_to_audio_rx: mpsc::Receiver<GuiToAudioMsg>,
    host_did_reset: bool,
    in_params: Vec<f32>,
    audio_player: AudioPlayer,
    did_init_preset: bool,
    bypassed: bool,
}

impl AudioThread {
    pub fn new(
        dsp_app: Box<dyn DSPBoxApp + Send>,
        gui_to_audio_rx: mpsc::Receiver<GuiToAudioMsg>,
    ) -> Self {
        Self {
            sample_rate: 0.0,
            dsp_app,
            gui_to_audio_rx,
            host_did_reset: false,
            in_params: Vec::new(),
            audio_player: AudioPlayer::new(),
            did_init_preset: false,
            bypassed: false,
        }
    }

    // called when host info has changed
    pub fn host_reset(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.host_did_reset = true;
    }

    // the main audio processing function (called from jack_thread.rs)
    pub fn process_audio_stereo<'a>(
        &mut self,
        in_l: &'a [f32],
        in_r: &'a [f32],
        out_l: &'a mut [f32],
        out_r: &'a mut [f32],
    ) {
        // only process once host_reset has been called at least once
        if self.sample_rate != 0.0 {
            if self.host_did_reset {
                self.dsp_app.host_reset(self.sample_rate);
                self.host_did_reset = false;
            }

            // assert all channels have the same number of frames
            let num_frames = out_l.len();
            assert_eq!(out_r.len(), num_frames);
            assert_eq!(in_l.len(), num_frames);
            assert_eq!(in_r.len(), num_frames);

            self.poll_gui_messages();

            if self.did_init_preset {
                let (in_l, in_r) = match self.audio_player.get_next(num_frames) {
                    Some(data) => data,
                    None => (in_l, in_r),
                };

                if self.bypassed {
                    out_l.copy_from_slice(in_l);
                    out_r.copy_from_slice(in_r);
                } else {
                    let mut proc_info = AudioProcessInfo {
                        in_l,
                        in_r,
                        out_l,
                        out_r,
                        sample_rate: self.sample_rate,
                        in_params: &self.in_params,
                    };

                    self.dsp_app.process_stereo(&mut proc_info);
                }
            }
        }
    }

    fn poll_gui_messages(&mut self) {
        for msg in self.gui_to_audio_rx.try_iter() {
            match msg {
                GuiToAudioMsg::ParamChanged(param) => {
                    self.in_params[param.id as usize] = param.value;
                }
                GuiToAudioMsg::InitPreset(params) => {
                    self.in_params.clear();
                    self.in_params.reserve_exact(params.len());
                    for param in params {
                        self.in_params.push(param.value);
                    }
                    self.did_init_preset = true;
                }
                GuiToAudioMsg::LoadAudioPlayerBuffer(buffer) => {
                    self.audio_player.load_buffer(buffer);
                }
                GuiToAudioMsg::Play => {
                    self.audio_player.play();
                }
                GuiToAudioMsg::Pause => {
                    self.audio_player.pause();
                }
                GuiToAudioMsg::Stop => {
                    self.audio_player.stop();
                }
                GuiToAudioMsg::Bypass => {
                    self.bypassed = true;
                }
                GuiToAudioMsg::Unbypass => {
                    self.bypassed = false;
                }
                GuiToAudioMsg::Panic => {}
            }
        }
    }
}
