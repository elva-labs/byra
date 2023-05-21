use std::fs::read;

use elva_byra_lib::output_writer::Sample;
use rumqttc::{AsyncClient, EventLoop, Key, MqttOptions, QoS, Transport};
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, mut eventloop, subject) = create_mqtt_client();

    task::spawn(publish_worker(client, subject, "tests/weight.json"));

    loop {
        let _ = eventloop.poll().await.unwrap();
    }
}

async fn publish_worker(client: AsyncClient, subject: String, integration_file: &str) {
    loop {
        // TODO: read from config
        match read(integration_file) {
            Ok(message) => {
                let d = String::from_utf8(message).unwrap();
                let message: Sample = serde_json::from_str(&d).unwrap();
                let res = client.publish(&subject, QoS::AtLeastOnce, false, d).await;

                match res {
                    Ok(_) => println!("Message delivered = {:?}", &message),
                    Err(e) => eprintln!("Failed to communicate with IOT core {:?}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to read data from file {:?} ", e)
            }
        }

        time::sleep(time::Duration::from_secs(10)).await;
    }
}

fn create_mqtt_client() -> (AsyncClient, EventLoop, String) {
    // TODO: read from config file
    let subject = "byra/weight".to_string();
    let ca = read("certs/AmazonRootCA1.pem").unwrap();
    let client_cert = read("certs/client.pem.crt").unwrap();
    let client_key = read("certs/client.pem.key").unwrap();
    let mut mqttoptions =
        MqttOptions::new("byra-01", "<target>.iot.eu-north-1.amazonaws.com", 8883);

    mqttoptions.set_transport(Transport::tls(
        ca,
        Some((client_cert, Key::RSA(client_key))),
        None,
    ));
    mqttoptions.set_keep_alive(time::Duration::from_secs(5));

    let client = AsyncClient::new(mqttoptions, 10);

    (client.0, client.1, subject)
}
