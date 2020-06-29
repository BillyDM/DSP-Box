extern crate iced;
extern crate iced_audio;
extern crate jack;
extern crate sndfile;

mod audio_player;
mod audio_thread;
mod blocks;
mod jack_client;
mod style;

use audio_thread::GuiToAudioMsg;
pub use audio_thread::{AudioProcessInfo, Param};

use blocks::{Block, KnobBlock, OptionKnobBlock};

use iced_audio::{DBRange, FloatRange, FreqRange, IntRange};

use iced::{
    button, executor, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Settings, Space, Subscription, Text, VerticalAlignment,
    time,
};

use std::sync::mpsc;
use std::time::Instant;

pub trait DSPBoxApp {
    fn host_reset(&mut self, sample_rate: f32);
    fn process_stereo(&mut self, proc_info: &mut AudioProcessInfo);
}

pub fn run(dsp_app: Box<dyn DSPBoxApp + std::marker::Send>, gui_setup: GuiSetup) {
    jack_client::run(dsp_app, gui_setup);
}

fn run_gui(gui_setup: GuiSetup, gui_to_audio_tx: mpsc::Sender<GuiToAudioMsg>) {
    DSPBoxGUI::run(Settings {
        antialiasing: true,
        flags: Flags {
            gui_setup,
            gui_to_audio_tx,
        },
        ..Settings::default()
    });
}

pub enum Range {
    Float(FloatRange),
    Int(IntRange),
    DB(DBRange),
    Freq(FreqRange),
}

impl Range {
    pub fn float(min: f32, max: f32) -> Self {
        Range::Float(FloatRange::new(min, max))
    }

    pub fn int(min: i32, max: i32) -> Self {
        Range::Int(IntRange::new(min, max))
    }

    pub fn db(min: f32, max: f32) -> Self {
        Range::DB(DBRange::new(min, max, iced_audio::Normal::center()))
    }

    pub fn freq(min: f32, max: f32) -> Self {
        Range::Freq(FreqRange::new(min, max))
    }
}

pub struct Knob {
    pub label: &'static str,
    pub value: f32,
    pub default_value: f32,
    pub range: Range,
}

pub struct OptionKnob {
    pub label: &'static str,
    pub value: u32,
    pub default_value: u32,
    pub options: Vec<String>,
}

struct Flags {
    pub gui_setup: GuiSetup,
    pub gui_to_audio_tx: mpsc::Sender<GuiToAudioMsg>,
}

impl Default for Flags {
    fn default() -> Self {
        let (tx, _) = mpsc::channel();

        Self {
            gui_setup: Default::default(),
            gui_to_audio_tx: tx,
        }
    }
}

pub struct GuiSetup {
    title: String,
    blocks: Vec<Block>,
    audio_file_path: Option<String>,
    audio_file_gain: f32,
    next_id: u32,
}

impl GuiSetup {
    pub fn new(title: &str) -> Self {
        Self {
            title: String::from(title),
            blocks: Vec::new(),
            audio_file_path: None,
            audio_file_gain: 1.0,
            next_id: 0,
        }
    }

    pub fn push_knob(&mut self, knob: Knob) {
        self.blocks.push(Block::Knob(KnobBlock::new(
            self.next_id,
            knob.value,
            knob.default_value,
            knob.label.clone(),
            knob.range,
        )));

        self.next_id += 1;
    }

    pub fn push_option_knob(&mut self, knob: OptionKnob) {
        self.blocks.push(Block::OptionKnob(OptionKnobBlock::new(
            self.next_id,
            knob.value as i32,
            knob.default_value as i32,
            knob.label.clone(),
            knob.options,
        )));

        self.next_id += 1;
    }

    pub fn load_audio_file(&mut self, path: &str, gain: f32) {
        self.audio_file_path = Some(String::from(path));
        self.audio_file_gain = gain;
    }
}

impl Default for GuiSetup {
    fn default() -> Self {
        Self {
            title: String::from("DSP Box"),
            blocks: Vec::new(),
            audio_file_path: None,
            audio_file_gain: 1.0,
            next_id: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Tick(Instant),
    ParamMoved(u32),
    PlayPauseBtnPressed,
    StopBtnPressed,
    BypassBtnPressed,
    PanicBtnPressed,
}

struct DSPBoxGUI {
    gui_setup: GuiSetup,
    current: Instant,
    theme: style::Theme,
    gui_to_audio_tx: mpsc::Sender<GuiToAudioMsg>,
    play_pause_btn: button::State,
    stop_btn: button::State,
    bypass_btn: button::State,
    panic_btn: button::State,
    play_pause_btn_stopped: bool,
    bypassed: bool,
    audio_file_loaded: bool,
}

impl DSPBoxGUI {
    pub fn update(&mut self, now: Instant) {
        self.current = now;
    }
}

impl Application for DSPBoxGUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn title(&self) -> String {
        self.gui_setup.title.clone()
    }

    fn new(flags: Flags) -> (Self, Command<Message>) {
        let mut init_preset: Vec<Param> = Vec::new();
        for block in flags.gui_setup.blocks.iter() {
            match block {
                Block::Knob(block) => block.add_params(&mut init_preset),
                Block::OptionKnob(block) => block.add_params(&mut init_preset),
            }
        }
        flags
            .gui_to_audio_tx
            .send(GuiToAudioMsg::InitPreset(init_preset))
            .unwrap();

        let mut audio_file_loaded = false;
        if let Some(audio_file_path) = &flags.gui_setup.audio_file_path {
            let audio_player_buffer = audio_player::AudioPlayerBuffer::new(
                &audio_file_path,
                4096,
                flags.gui_setup.audio_file_gain,
            );

            match audio_player_buffer {
                Ok(buffer) => {
                    audio_file_loaded = true;

                    flags
                        .gui_to_audio_tx
                        .send(GuiToAudioMsg::LoadAudioPlayerBuffer(buffer))
                        .unwrap();
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            }
        }

        (
            Self {
                gui_setup: flags.gui_setup,
                current: Instant::now(),
                theme: style::Theme::Dark,
                gui_to_audio_tx: flags.gui_to_audio_tx,
                play_pause_btn: button::State::new(),
                stop_btn: button::State::new(),
                bypass_btn: button::State::new(),
                panic_btn: button::State::new(),
                play_pause_btn_stopped: true,
                bypassed: false,
                audio_file_loaded,
            },
            Command::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(10)).map(|instant| Message::Tick(instant))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(instant) => {
                self.update(instant);

                // Normally you would animate the meter here, but basic
                // knobs are used instead for demonstration.
            }
            Message::ParamMoved(_id) => {
                for block in self.gui_setup.blocks.iter_mut() {
                    match block {
                        Block::Knob(block) => block.update(message, &self.gui_to_audio_tx),
                        Block::OptionKnob(block) => block.update(message, &self.gui_to_audio_tx),
                    }
                }
            }
            Message::PlayPauseBtnPressed => {
                if self.audio_file_loaded {
                    if self.play_pause_btn_stopped {
                        self.gui_to_audio_tx.send(GuiToAudioMsg::Play).unwrap();
                    } else {
                        self.gui_to_audio_tx.send(GuiToAudioMsg::Pause).unwrap();
                    }

                    self.play_pause_btn_stopped = !self.play_pause_btn_stopped;
                }
            }
            Message::StopBtnPressed => {
                if self.audio_file_loaded {
                    self.play_pause_btn_stopped = true;
                    self.gui_to_audio_tx.send(GuiToAudioMsg::Stop).unwrap();
                }
            }
            Message::BypassBtnPressed => {
                if self.bypassed {
                    self.gui_to_audio_tx.send(GuiToAudioMsg::Unbypass).unwrap();
                } else {
                    self.gui_to_audio_tx.send(GuiToAudioMsg::Bypass).unwrap();
                }

                self.bypassed = !self.bypassed;
            }
            Message::PanicBtnPressed => {
                self.play_pause_btn_stopped = true;
                self.gui_to_audio_tx.send(GuiToAudioMsg::Panic).unwrap();
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let play_pause_btn = if self.audio_file_loaded {
            Button::new(
                &mut self.play_pause_btn,
                Text::new(if self.play_pause_btn_stopped {
                    "Play"
                } else {
                    "Pause"
                })
                .size(16)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::from(50))
            .on_press(Message::PlayPauseBtnPressed)
            .style(self.theme.button())
        } else {
            Button::new(
                &mut self.play_pause_btn,
                Text::new("Play")
                .size(16)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::from(50))
            .style(self.theme.disabled_button())
        };

        let stop_btn = if self.audio_file_loaded {
            Button::new(
                &mut self.stop_btn,
                Text::new("Stop")
                    .size(16)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::from(40))
            .on_press(Message::StopBtnPressed)
            .style(self.theme.button())
        } else {
            Button::new(
                &mut self.stop_btn,
                Text::new("Stop")
                    .size(16)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::from(40))
            .style(self.theme.disabled_button())
        };

        let bypass_btn = Button::new(
            &mut self.bypass_btn,
            Text::new("Bypass")
                .size(16)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
        .width(Length::from(60))
        .on_press(Message::BypassBtnPressed)
        .style(if self.bypassed {
            self.theme.bypassed_button()
        } else {
            self.theme.button()
        });

        let panic_btn = Button::new(
            &mut self.panic_btn,
            Text::new("Panic")
                .size(16)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
        .width(Length::from(50))
        .on_press(Message::PanicBtnPressed)
        .style(self.theme.button());

        let mut blocks: Vec<Element<Message>> = Vec::new();
        blocks.reserve_exact(self.gui_setup.blocks.len());

        for block in self.gui_setup.blocks.iter_mut() {
            blocks.push(match block {
                Block::Knob(block) => block.view(&self.theme),
                Block::OptionKnob(block) => block.view(&self.theme),
            });
        }

        let top_bar = Container::new(
            Row::new()
                .width(Length::Fill)
                .height(Length::from(40))
                .padding(7)
                .spacing(7)
                .push(play_pause_btn)
                .push(stop_btn)
                .push(Space::with_width(Length::Fill))
                .push(bypass_btn)
                .push(panic_btn),
        )
        .padding(0)
        .style(self.theme.top_bar_container());

        let block_row = Row::with_children(blocks).spacing(3).padding(8);

        let block_container = Container::new(block_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        let content = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .spacing(0)
            .push(top_bar)
            .push(block_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(self.theme.page_container())
            .into()
    }
}

/*
mod time {
    use iced::futures;
    use std::time::Instant;

    pub fn every(duration: std::time::Duration) -> iced::Subscription<Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| Instant::now())
                .boxed()
        }
    }
}
*/
