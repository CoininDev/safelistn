use jack::*;

fn main() {
    // the limit of our sound
    let threshold = 0.1;

    // initializing client
    let (client, _status) = Client::new("safelistn", ClientOptions::NO_START_SERVER).unwrap();

    // creating its ports
    let in_port_l = client.register_port("inL", AudioIn::default()).unwrap();
    let in_port_r = client.register_port("inR", AudioIn::default()).unwrap();
    let mut out_port_l = client.register_port("outL", AudioOut::default()).unwrap();
    let mut out_port_r = client.register_port("outR", AudioOut::default()).unwrap();

    // processing audio (audio compression)
    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_pl = in_port_l.as_slice(ps);
            let in_pr = in_port_r.as_slice(ps);
            let out_pl = out_port_l.as_mut_slice(ps);
            let out_pr = out_port_r.as_mut_slice(ps);

            for (i, (in_l, in_r)) in in_pl.iter().zip(in_pr.iter()).enumerate() {
                let mut out_l = *in_l;
                let mut out_r = *in_r;

                if out_l > threshold {
                    out_l = threshold + (out_l - threshold) / 2.0;
                } else if out_l < -threshold {
                    out_l = -threshold + (out_l + threshold) / 2.0;
                }

                if out_r > threshold {
                    out_r = threshold + (out_r - threshold) / 2.0;
                } else if out_r < -threshold {
                    out_r = -threshold + (out_r + threshold) / 2.0;
                }

                out_pl[i] = out_l;
                out_pr[i] = out_r;
            }

            jack::Control::Continue
        },
    );

    // activating client
    let active_client = client.activate_async((), process).unwrap();


    arrange_connections(&active_client.as_client());
    // Mantendo o cliente ativo
    std::thread::park();

    // Deactivating the client
    active_client.deactivate().unwrap();
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