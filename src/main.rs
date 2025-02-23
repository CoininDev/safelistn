use jack::*;
use console::Term;

struct Compressor {
    threshold: f32,
    ratio: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    makeup_gain: f32
}

impl Compressor {
    fn new(threshold:f32, ratio:f32, attack_ms:f32, release_ms:f32, sample_rate:f32, makeup_gain:f32) -> Self {
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

        Self {
            threshold,
            ratio,
            attack_coeff,
            release_coeff,
            envelope: 0.0,
            makeup_gain
        }
    }

    fn update_envelope(&mut self, input_abs:f32) {
        if input_abs > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * input_abs;
        }
        else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * input_abs;
        }
    }

    fn calculate_gain(&self) -> f32{
        let mut gain = 1.0;
        if self.envelope > self.threshold {
            let excess = self.envelope - self.threshold;
            gain = (self.threshold + excess / self.ratio) / self.envelope;
        }
        gain * self.makeup_gain
    }
}

fn main() {

    // initializing client
    let (client, _status) = Client::new("safelistn", ClientOptions::NO_START_SERVER).unwrap();

    // creating its ports
    let in_port_l = client.register_port("inL", AudioIn::default()).unwrap();
    let in_port_r = client.register_port("inR", AudioIn::default()).unwrap();
    let mut out_port_l = client.register_port("outL", AudioOut::default()).unwrap();
    let mut out_port_r = client.register_port("outR", AudioOut::default()).unwrap();

    // Initializing compressor
    let sample_rate = client.sample_rate() as f32;
    let mut compressor = Compressor::new(
        0.2,
        4.0,
        5.0,
        50.0,
        sample_rate,
        3.0
    );

    // processing audio (audio compression)
    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_pl = in_port_l.as_slice(ps);
            let in_pr = in_port_r.as_slice(ps);
            let out_pl = out_port_l.as_mut_slice(ps);
            let out_pr = out_port_r.as_mut_slice(ps);

            for (((in_l, in_r), out_l), out_r ) in in_pl.iter()
                .zip(in_pr.iter())
                .zip(out_pl.iter_mut())
                .zip(out_pr.iter_mut())
            {
                let in_l_val = *in_l;
                let in_r_val = *in_r;

                let max_input = in_l_val.abs().max(in_r_val.abs());

                compressor.update_envelope(max_input);
                let gain = compressor.calculate_gain();

                *out_l = in_l_val * gain;
                *out_r = in_r_val * gain;
            }

            jack::Control::Continue
        },
    );

    // activating client
    let active_client = client.activate_async((), process).unwrap();


    arrange_connections(&active_client.as_client());

    // Keep the client running until the user presses 'q'
    println!("Press 'q' to exit");
    let term = Term::stdout();
    loop {
        if let Ok('q') = term.read_char() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Deactivate the client
    disarrange_connections(&active_client.as_client());
    active_client.deactivate().unwrap();
}

fn disarrange_connections(client: &Client) {
    // Getting ports connected to the client
    let connectionsl = client.port_by_name("safelistn:inL").unwrap().get_connections();
    let connectionsr = client.port_by_name("safelistn:inR").unwrap().get_connections();

    // Redirect them back to system
    connectionsl
        .iter()
        .for_each(|conn| {
            let _ = client.disconnect_ports_by_name(conn, "safelistn:inL");
            let _ = client.connect_ports_by_name(conn, "system:playback_1");
        })
    ;

    // I suspect that sometimes the code don't get enough time to do it correctly, so I think it should work.
    std::thread::sleep(std::time::Duration::from_millis(100));

    connectionsr
        .iter()
        .for_each(|conn| {
            let _ = client.disconnect_ports_by_name(conn, "safelistn:inR");
            let _ = client.connect_ports_by_name(conn, "system:playback_2");
        })
    ;

    std::thread::sleep(std::time::Duration::from_millis(100));

    //disconnect client to system
    let _ = client.disconnect_ports_by_name("safelistn:outL", "system:playback_1");
    let _ = client.disconnect_ports_by_name("safelistn:outR", "system:playback_2");
}

fn arrange_connections(client: &Client) {
    // Getting ports connected to main output
    let system_port = client.ports(Some("system:playback_"), None, PortFlags::empty());
    let sysl_connections = client.port_by_name(system_port[0].as_str()).unwrap().get_connections();
    let sysr_connections = client.port_by_name(system_port[1].as_str()).unwrap().get_connections();

    // Redirect them to our client 
    sysl_connections
        .iter()
        .for_each(|conn| {
            let _ = client.disconnect_ports_by_name(conn, "system:playback_1");
            let _ = client.connect_ports_by_name(conn, "safelistn:inL");
        });
    sysr_connections
        .iter()
        .for_each(|conn| {
            let _ = client.disconnect_ports_by_name(conn, "system:playback_2");
            let _ = client.connect_ports_by_name(conn, "safelistn:inR");
        });
    
    // Connect client to system
    let _ = client.connect_ports_by_name("safelistn:outL", "system:playback_1");
    let _ = client.connect_ports_by_name("safelistn:outR", "system:playback_2");
}