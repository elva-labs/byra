//! The byra-scale is used to read values from load cells (in conjunction with the HX711 module).
//! This binary has only been tested on the raspberry pi zero (w).
//!
//! ## Calibrate
//! This command samples reads from the load cells and then outputs the average rate output
//! for both 0kg and 1kg load.
//!
//! ```bash
//! elva-byra-scale --calibrate
//! ```
//!
//! ## Run
//! Start a long lived process, readings are pushed to stdout or file (based on given settings).  
//!
//! ```bash
//! elva-byra # Reads settings from `~/.config/byra/settings.toml` by default.
//!
//! elva-byra --help
//! ```
//!
//! ## Example config
//! ```toml
//! # ~/.config/byra/settings.toml
//! dout = 23
//! dt_sck = 24
//! offset = 521703
//! calibration = 545351
//! backoff = 3
//! retry = 3
//!```

use clap::Parser;
use cli_config::ServiceConfig;
use elva_byra_lib::output_writer::write_weight_to_sock;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::error::Error;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod cli_config;
mod init;

use crate::cli_config::Args;
use crate::init::bootstrap;
use elva_byra_lib::hx711::HX711;

static MODULE: &str = "HX711";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (settings, byra) = bootstrap(&args).expect("Failed to init byra scale");
    let mut byra = byra;

    SimpleLogger::new()
        .with_level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Warn,
        })
        .init()
        .unwrap();
    info!("Starting byra-scale, setting up gpio & performing hx711 reset");
    byra.reset();
    info!("{MODULE} reset complete");

    if args.calibrate {
        // TODO: pass N via cli
        let result = byra.calibrate(10);

        info!("\roffset={}\ncalibration={}", result.0, result.1);

        return Ok(());
    }

    thread::sleep(Duration::from_secs(1));

    let (tx, rx) = channel::<f32>();
    let server_cfg = settings.clone();

    thread::spawn(move || {
        server(rx, server_cfg).unwrap();
    });

    loop {
        let mut last_value = 0_f32;

        {
            match byra.read() {
                Ok(v) => {
                    last_value = v;
                    tx.send(v).unwrap();
                }
                Err(e) => error!("Failed to update scale reading {}", e),
            }
        }

        info!("current value={}", last_value);
        thread::sleep(Duration::from_secs(settings.backoff));
    }
}

fn server(last_read: Receiver<f32>, settings: ServiceConfig) -> Result<(), Box<dyn Error>> {
    info!("Booting Publisher");
    match std::fs::remove_file("byra.sock") {
        Ok(_) => debug!("Removed old socket file"),
        Err(_) => debug!("No previous socket file exist"),
    };

    let clients: Arc<Mutex<HashMap<usize, UnixStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let server_stream = match UnixListener::bind("byra.sock") {
        Err(_) => panic!("failed to bind socket"),
        Ok(stream) => stream,
    };
    let (tx, rx) = std::sync::mpsc::channel::<usize>();
    let a_clients = clients.clone();

    // Publish loop
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(settings.backoff));

        {
            let next_reading = last_read.recv().unwrap();
            let mut message_lock = a_clients.lock().expect("Failed to lock client list");

            info!("Notifying listeners n={}", message_lock.len());

            for (id, connection) in message_lock.iter_mut() {
                match write_weight_to_sock(next_reading, connection) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to communicate with client {:?}, dropping client", e);
                        connection.shutdown(Shutdown::Both).unwrap_or_default();
                        tx.send(*id).unwrap();
                    }
                }
            }
        }
    });

    let b_clients = clients.clone();

    // Remove dangling connections
    thread::spawn(move || loop {
        let connection_id = rx.recv().unwrap();

        {
            let mut d = b_clients.lock().unwrap();
            d.remove(&connection_id);
            info!("Removed connection {}", connection_id);
        }
    });

    // Client connection loop
    loop {
        let (client, _addr) = match server_stream.accept() {
            Ok(s) => s,
            Err(e) => {
                error!(
                    "Failed to establish connection with incoming listener {:?}",
                    e
                );
                continue;
            }
        };

        {
            let mut clients = clients
                .lock()
                .expect("Failed to receive a lock on client list");
            let size = clients.len() + 1;

            debug!("Registered client {}", size);
            clients.insert(size, client);
        }
    }
}
