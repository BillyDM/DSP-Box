use sndfile::SndFileError;

pub struct AudioPlayerBuffer {
    data_l: Vec<f32>,
    data_r: Vec<f32>,
}

impl AudioPlayerBuffer {
    pub fn new(path: &str, buffer_size: usize, gain: f32) -> Result<Self, SndFileError> {
        use sndfile::*;

        let mut snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(path)?;

        let data: Vec<f32> = match snd.read_all_to_vec() {
            Ok(f) => f,
            Err(_) => {
                return Err(SndFileError::InternalError(format!(
                    "error reading data in audio file: {}",
                    path
                )))
            }
        };

        let sample_rate = snd.get_samplerate();

        let n_frames = match snd.len() {
            Ok(n) => n,
            Err(_) => {
                return Err(SndFileError::InternalError(format!(
                    "error reading length of audio file: {}",
                    path
                )))
            }
        };

        if n_frames < buffer_size as u64 {
            return Err(SndFileError::InvalidParameter(format!(
                "error: audio file is shorter than the maximum buffer length of {}: {}",
                buffer_size, path
            )));
        }

        let n_channels = snd.get_channels();

        let (mut data_l, mut data_r) = if n_channels == 1 {
            (data.clone(), data)
        } else if n_channels == 2 {
            Self::stereo_deinterleave(&data, n_frames as usize)
        } else {
            return Err(SndFileError::InvalidParameter(format!(
                "error: audio file is not mono or stereo: {}",
                path
            )));
        };

        if gain != 1.0 {
            for smp in data_l.iter_mut() {
                *smp *= gain;
            }
            for smp in data_r.iter_mut() {
                *smp *= gain;
            }
        }

        println!("Loaded audio file `{}`:", path);
        println!(
            "  Length: {:.2} seconds",
            n_frames as f64 / sample_rate as f64
        );
        println!("  Sample rate: {} Hz", sample_rate);
        println!("  Channel count: {}", n_channels);

        Ok(Self { data_l, data_r })
    }

    fn stereo_deinterleave(data: &Vec<f32>, n_frames: usize) -> (Vec<f32>, Vec<f32>) {
        assert_eq!(data.len(), n_frames * 2);

        let mut l: Vec<f32> = Vec::new();
        let mut r: Vec<f32> = Vec::new();

        l.reserve_exact(n_frames);
        r.reserve_exact(n_frames);

        let mut data = &data[..];
        while data.len() >= 2 {
            l.push(data[0]);
            r.push(data[1]);

            data = &data[2..];
        }

        (l, r)
    }
}

pub struct AudioPlayer {
    transport: usize,
    buffer: Option<AudioPlayerBuffer>,
    playing: bool,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            transport: 0,
            buffer: None,
            playing: false,
        }
    }

    pub fn load_buffer(&mut self, buffer: AudioPlayerBuffer) {
        self.buffer = Some(buffer);
    }

    pub fn _clear_buffer(&mut self) {
        self.buffer = None;
        self.playing = false;
        self.transport = 0;
    }

    pub fn play(&mut self) {
        if !self.playing {
            if let Some(_) = self.buffer {
                self.playing = true;
            }
        }
    }

    pub fn pause(&mut self) {
        if self.playing {
            self.playing = false;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.transport = 0;
    }

    pub fn get_next(&mut self, num_frames: usize) -> Option<(&[f32], &[f32])> {
        if self.playing {
            if let Some(buffer) = &self.buffer {
                assert!(buffer.data_l.len() >= num_frames);

                if self.transport + num_frames >= buffer.data_l.len() {
                    self.transport = 0;
                }

                let transport = self.transport;
                self.transport += num_frames;

                Some((
                    &buffer.data_l[transport..self.transport],
                    &buffer.data_r[transport..self.transport],
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}
