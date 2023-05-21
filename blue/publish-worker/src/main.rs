use std::fs::read;

use config::Config;
use elva_byra_lib::output_writer::Sample;
use log::{error, info};
use rumqttc::{AsyncClient, EventLoop, Key, MqttOptions, QoS, Transport};
use simple_logger::SimpleLogger;
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (settings, client, mut eventloop) = bootstrap();

    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("Failed to init logger");
    task::spawn(publish_worker(client, settings));

    loop {
        let _ = eventloop.poll().await.unwrap();
    }
}

async fn publish_worker(client: AsyncClient, settings: Settings) {
    let mut previous_sample: Option<Sample> = None;

    loop {
        time::sleep(time::Duration::from_secs(settings.publish_interval_sec)).await;
        let integration_file = match read(&settings.integration_file) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to read from integration file {:?} ", e);
                continue;
            }
        };
        let raw_message = match String::from_utf8(integration_file) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to read data from integration file {:?}", e);
                continue;
            }
        };
        let sample: Sample = match serde_json::from_str(&raw_message) {
            Ok(s) => s,
            Err(e) => {
                error!(
                    "Failed to parse data from integration file to a Sample {:?}",
                    e
                );
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
            .publish(&settings.subject, QoS::AtLeastOnce, false, raw_message)
            .await
        {
            Ok(_) => {
                info!("Message delivered = {:?}", &sample);
                previous_sample = Some(sample);
            }
            Err(e) => error!("Failed to communicate with IOT core {:?}", e),
        }
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
    publish_interval_sec: u64,
    publish_on_diff_gram: f32,
    integration_file: String,
}
