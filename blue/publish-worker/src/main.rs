use std::fs::read;

use config::Config;
use elva_byra_lib::output_writer::Sample;
use log::{error, info};
use rumqttc::{AsyncClient, EventLoop, Key, MqttOptions, QoS, Transport};
use simple_logger::SimpleLogger;
use tokio::{
    io::{AsyncReadExt, Interest},
    net::UnixStream,
    task, time,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("Failed to init logger");

    info!("Bootstrapping configuration");

    let (settings, client, mut eventloop) = bootstrap();

    task::spawn(async move {
        let mut stream = UnixStream::connect("byra.sock")
            .await
            .expect("Failed to connect to byra.sock");

        let mut previous_sample: Option<Sample> = None;
        let mut buff = vec![0; 512];

        loop {
            let ready = stream.ready(Interest::READABLE).await.unwrap();

            if !ready.is_readable() {
                continue;
            }

            let message_len = stream.read(&mut buff).await.unwrap();
            let message = String::from_utf8_lossy(&buff[0..message_len]).to_string();
            let sample: Sample = match serde_json::from_str(&message) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to parse message {:?}", e);
                    continue;
                }
            };

            if let Some(previous_sample) = &previous_sample {
                if (previous_sample.grams - sample.grams).abs() < settings.publish_on_diff_gram {
                    info!("Skipping publish event, sample diff is too small");
                    continue;
                }
            }

            match client
                .publish(&settings.subject, QoS::AtLeastOnce, false, message)
                .await
            {
                Ok(_) => {
                    info!("Message delivered = {:?}", &sample);
                    previous_sample = Some(sample);
                }
                Err(e) => error!("Failed to communicate with IOT core {:?}", e),
            }
        }
    });

    loop {
        let _ = eventloop.poll().await.unwrap();
    }
}

/// Reads runtime settings from file & creates a new MQTT client
/// panics on any type of settings failure.
fn bootstrap() -> (Settings, AsyncClient, EventLoop) {
    let settings = Config::builder()
        .add_source(config::File::with_name("tests/config.toml"))
        .build()
        .expect("Failed to read boot configuration")
        .try_deserialize::<Settings>()
        .expect("Failed to parse settings config to struct");
    let (client, eventloop) = create_mqtt_client(&settings);

    (settings, client, eventloop)
}

fn create_mqtt_client(settings: &Settings) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(
        &settings.device_id,
        &settings.endpoint,
        settings.endpoint_port,
    );
    let client_cert = read(&settings.client_cert).expect("Failed to read client certificate");
    let client_key = read(&settings.client_key).expect("Failed to read client (private) key");
    let ca = read(&settings.ca).expect("Failed to read certificate authority");

    mqttoptions.set_transport(Transport::tls(
        ca,
        Some((client_cert, Key::RSA(client_key))),
        None,
    ));
    mqttoptions.set_keep_alive(time::Duration::from_secs(5));

    AsyncClient::new(mqttoptions, 10)
}

#[derive(serde::Deserialize, Debug)]
struct Settings {
    device_id: String,
    subject: String,
    ca: String,
    client_cert: String,
    client_key: String,
    endpoint: String,
    endpoint_port: u16,
    publish_on_diff_gram: f32,
}
