/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: AGPL-3.0-only
* Refer to the file "LICENSE" for details.
*******************************************************************************/

struct Handler {
    port_name_pairs: Vec<(String, String)>,
}

impl jack::NotificationHandler for Handler {
    fn thread_init(&self, c: &jack::Client) {
        //During startup, make sure that our ports get connected.
        self.converge(c);
    }

    fn graph_reorder(&mut self, c: &jack::Client) -> jack::Control {
        //Whenever the client/port configuration changes in JACK, make sure that our ports are
        //still connected.
        self.converge(c);
        jack::Control::Continue
    }
}

impl Handler {
    fn converge(&self, c: &jack::Client) {
        //for each port pair...
        for (src, dst) in self.port_name_pairs.iter() {
            //check if those ports exist
            let src_port = match c.port_by_name(src) {
                Some(port) => port,
                None => continue,
            };
            if c.port_by_name(dst).is_none() {
                continue;
            };

            //check if we have anything to do
            let is_connected = match src_port.is_connected_to(dst) {
                Ok(val) => val,
                Err(e) => {
                    println!("unexpected error while checking {} -> {}: {}", src, dst, e);
                    continue;
                }
            };
            if is_connected {
                continue;
            }

            //connect if necessary
            match c.connect_ports_by_name(src, dst) {
                Ok(()) => {
                    println!("connected {} -> {}", src, dst);
                }
                Err(jack::Error::PortAlreadyConnected(_, _)) => {}
                Err(jack::Error::PortConnectionError(_, _)) => {
                    println!("could not connect {} -> {}", src, dst);
                }
                Err(e) => {
                    println!(
                        "unexpected error while connecting {} -> {}: {}",
                        src, dst, e
                    );
                }
            }
        }
    }
}

fn main() {
    let mut all_args = std::env::args();
    let program = all_args.next().unwrap();
    let args = all_args.collect::<Vec<String>>();

    let mut opts = getopts::Options::new();
    const SRC_CLIENT_HELP: &str = "name of JACK client owning the source ports";
    const DST_CLIENT_HELP: &str = "name of JACK client owning the destination ports";
    const SRC_PORT_HELP: &str =
        "name of source port (give more than once to connect multiple ports)";
    const DST_PORT_HELP: &str =
        "name of destination port (give more than once to connect multiple ports)";
    opts.reqopt("f", "from-client", SRC_CLIENT_HELP, "NAME");
    opts.reqopt("t", "to-client", DST_CLIENT_HELP, "NAME");
    opts.optmulti("F", "from-port", SRC_PORT_HELP, "NAME");
    opts.optmulti("T", "to-port", DST_PORT_HELP, "NAME");
    opts.optflag("h", "help", "show this message");

    //need to check for --help before opts.parse(), otherwise we will probably fail because some
    //required options are missing
    if args.iter().any(|s| s == "-h" || s == "--help") {
        let desc = "Ensures that a certain set of JACK ports are always connected to each other (if they are present).";
        let brief = format!("Usage: {} [options]\n\n{}", program, desc);
        print!("{}", opts.usage(&brief));
        return;
    }

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} (try --help)", e.to_string());
            return;
        }
    };
    let src_client_name = matches.opt_str("from-client").unwrap();
    let dst_client_name = matches.opt_str("to-client").unwrap();
    let src_port_names = prepend_client(&src_client_name, matches.opt_strs("from-port"));
    let dst_port_names = prepend_client(&dst_client_name, matches.opt_strs("to-port"));

    if src_port_names.len() != dst_port_names.len() {
        eprintln!(
            "error: number of source ports ({}) must be equal to number of destination ports ({})",
            src_port_names.len(),
            dst_port_names.len(),
        );
        return;
    }
    let port_name_pairs = src_port_names.into_iter().zip(dst_port_names).collect();

    //run a handler that continuously maintains the desired port connections
    let (raw_client, _) =
        jack::Client::new("jackautoplug", jack::ClientOptions::NO_START_SERVER).unwrap();
    let handler = Handler { port_name_pairs };
    let client = raw_client.activate_async(handler, ()).unwrap();

    std::thread::park();
    client.deactivate().unwrap(); //NOTE: unreachable, but silences clippy
}

fn prepend_client(client_name: &str, port_names: Vec<String>) -> Vec<String> {
    port_names
        .iter()
        .map(|s| format!("{}:{}", client_name, &s))
        .collect()
}
