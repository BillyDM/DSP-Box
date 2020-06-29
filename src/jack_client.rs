use std::sync::{mpsc, Arc, Mutex};

use crate::audio_thread::{AudioThread, GuiToAudioMsg};
use crate::{DSPBoxApp, GuiSetup};

pub fn run(dsp_app: Box<dyn DSPBoxApp + std::marker::Send>, gui_setup: GuiSetup) {
    let (gui_to_audio_tx, gui_to_audio_rx) = mpsc::channel::<GuiToAudioMsg>();

    let audio_thread = Arc::new(Mutex::new(AudioThread::new(dsp_app, gui_to_audio_rx)));

    // Create client
    let (client, _status) =
        jack::Client::new("dsp_box", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_l = client
        .register_port("dsp_box_in_l", jack::AudioIn::default())
        .unwrap();
    let in_r = client
        .register_port("dsp_box_in_r", jack::AudioIn::default())
        .unwrap();
    let mut out_l = client
        .register_port("dsp_box_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_r = client
        .register_port("dsp_box_out_r", jack::AudioOut::default())
        .unwrap();
    let in_l_name = in_l.name().unwrap();
    let in_r_name = in_r.name().unwrap();
    let out_l_name = out_l.name().unwrap();
    let out_r_name = out_r.name().unwrap();

    // create a seperate thread for the audio processing
    let audio_thread_arc = Arc::clone(&audio_thread);
    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let mut audio_thread = audio_thread_arc.lock().unwrap();

        audio_thread.process_audio_stereo(
            in_l.as_slice(ps),
            in_r.as_slice(ps),
            out_l.as_mut_slice(ps),
            out_r.as_mut_slice(ps),
        );

        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client
        .activate_async(JackNotifications::new(Arc::clone(&audio_thread)), process)
        .unwrap();

    // find system audio outputs
    let out_ports: Vec<String> = active_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_INPUT,
    );
    println!("physical out ports: {:?}", out_ports);
    if out_ports.len() < 2 {
        panic!("Not enough physical audio outputs, need at least {}", 2);
    }

    // find system audio inputs
    let in_ports: Vec<String> = active_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_OUTPUT,
    );
    println!("physical in ports: {:?}", in_ports);

    // connect the ports we created to system output
    active_client
        .as_client()
        .connect_ports_by_name(&out_l_name, &out_ports[0])
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name(&out_r_name, &out_ports[1])
        .unwrap();
    if in_ports.len() >= 1 {
        active_client
            .as_client()
            .connect_ports_by_name(&in_ports[0], &in_l_name)
            .unwrap();
    }
    if in_ports.len() >= 2 {
        active_client
            .as_client()
            .connect_ports_by_name(&in_ports[1], &in_r_name)
            .unwrap();
    }

    // run the gui thread until app is closed
    crate::run_gui(gui_setup, gui_to_audio_tx);

    // shut down jack client
    active_client.deactivate().unwrap();
}

// handles jack notifications
struct JackNotifications {
    audio_thread_arc: Arc<Mutex<AudioThread>>,
}

impl JackNotifications {
    pub fn new(audio_thread_arc: Arc<Mutex<AudioThread>>) -> Self {
        JackNotifications { audio_thread_arc }
    }
}

impl jack::NotificationHandler for JackNotifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("Jack: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);

        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        {
            let mut audio_thread = self.audio_thread_arc.lock().unwrap();
            audio_thread.host_reset(srate as f32);
        }
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
