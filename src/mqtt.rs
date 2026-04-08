use core::num::NonZeroU16;
use embassy_executor::Spawner;
use embassy_net::{IpEndpoint, Stack, tcp::TcpSocket};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use rust_mqtt::{
    buffer::AllocBuffer,
    client::{
        Client,
        event::Event,
        options::{ConnectOptions, SubscriptionOptions},
    },
    config::{KeepAlive, SessionExpiryInterval},
    types::{MqttString, TopicName},
};
use smoltcp::wire::{IpAddress, Ipv4Address};

use crate::display::{DISPLAY_CHANNEL, DisplayEvent};

const TCP_BUFFER_SIZE: usize = 1024;
const MQTT_MSG_MAX_LEN: usize = 256;
const KEEP_ALIVE_INTERVAL_SECS: u16 = 60;
const RECONNECT_DELAY_MS: u64 = 5000;

#[derive(Clone, Copy, Debug)]
struct MqttMessage {
    data: [u8; MQTT_MSG_MAX_LEN],
    len: usize,
}

pub struct Notification {
    pub service: ServiceType,
    pub count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum ServiceType {
    Slack,
    Gmail,
    GoogleCalendar,
}

static DECODE_CHANNEL: Channel<CriticalSectionRawMutex, MqttMessage, 4> = Channel::new();

#[embassy_executor::task]
pub async fn decode_task() {
    loop {
        let msg = DECODE_CHANNEL.receive().await;
        defmt::info!("Decoding MQTT message");
        if let Some(notification) = decode_mqtt_message(&msg) {
            DISPLAY_CHANNEL
                .send(DisplayEvent::Notification(notification))
                .await;
        }
    }
}

#[embassy_executor::task]
pub async fn mqtt_task(stack: Stack<'static>) {
    let (addr, port, client_id, topic_str) = load_mqtt_config();

    stack.wait_config_up().await;
    defmt::info!("MQTT: network is up, starting MQTT client");

    let mut buffer = AllocBuffer;

    loop {
        let mut tcp_rx_buffer = [0u8; TCP_BUFFER_SIZE];
        let mut tcp_tx_buffer = [0u8; TCP_BUFFER_SIZE];
        let mut socket = TcpSocket::new(stack, &mut tcp_rx_buffer, &mut tcp_tx_buffer);

        let remote = IpEndpoint::new(IpAddress::Ipv4(addr), port);

        if let Err(e) = socket.connect(remote).await {
            defmt::error!("MQTT: TCP connect failed: {:?}", e);
            Timer::after_millis(RECONNECT_DELAY_MS).await;
            continue;
        }

        defmt::info!("MQTT: TCP connected");

        let connect_options = ConnectOptions::new()
            .clean_start()
            .session_expiry_interval(SessionExpiryInterval::NeverEnd)
            .keep_alive(KeepAlive::Seconds(
                NonZeroU16::new(KEEP_ALIVE_INTERVAL_SECS).unwrap(),
            ));

        let mut client: Client<'_, _, _, 1, 1, 1, 0> = Client::new(&mut buffer);

        if let Err(e) = client
            .connect(
                &mut socket,
                &connect_options,
                Some(MqttString::from_str(client_id).unwrap()),
            )
            .await
        {
            defmt::error!("MQTT: connect failed: {:?}", e);
            Timer::after_millis(RECONNECT_DELAY_MS).await;
            continue;
        }

        defmt::info!("MQTT: connected to broker");

        let topic = TopicName::new(MqttString::from_str(topic_str).unwrap()).unwrap();

        if let Err(e) = client
            .subscribe(topic.as_borrowed().into(), SubscriptionOptions::new())
            .await
        {
            defmt::error!("MQTT: subscribe failed: {:?}", e);
            Timer::after_millis(RECONNECT_DELAY_MS).await;
            continue;
        }

        defmt::info!("MQTT: subscribed to {}", topic_str);

        loop {
            match client.poll().await {
                Ok(event) => {
                    if let Event::Publish(publish) = event {
                        defmt::info!("MQTT: received message on topic");
                        let bytes = publish.message.as_bytes();
                        let len = bytes.len().min(MQTT_MSG_MAX_LEN);
                        let mut msg = MqttMessage {
                            data: [0u8; MQTT_MSG_MAX_LEN],
                            len,
                        };
                        msg.data[..len].copy_from_slice(&bytes[..len]);
                        DECODE_CHANNEL.send(msg).await;
                    }
                }
                Err(e) => {
                    defmt::error!("MQTT: poll error: {:?}", e);
                    break;
                }
            }
        }

        client.abort().await;
        defmt::warn!("MQTT: disconnected, reconnecting...");
        Timer::after_millis(RECONNECT_DELAY_MS).await;
    }
}

pub async fn spawn_tasks(spawner: &Spawner, stack: Stack<'static>) {
    spawner.spawn(mqtt_task(stack)).unwrap();
    spawner.spawn(decode_task()).unwrap();
}

fn load_mqtt_config() -> (Ipv4Address, u16, &'static str, &'static str) {
    let mut host_ip = "";
    let mut host_port = 0u16;
    let mut client_id = "";
    let mut topic = "";

    let mqtt_config = include_str!("../.mqtt.config");
    for line in mqtt_config.lines() {
        if let Some(stripped) = line.strip_prefix("HOST_IP_ADDRESS=") {
            host_ip = stripped.trim_matches('"');
        } else if let Some(stripped) = line.strip_prefix("HOST_PORT=") {
            host_port = stripped.trim().parse().unwrap_or(0);
        } else if let Some(stripped) = line.strip_prefix("CLIENT_ID=") {
            client_id = stripped.trim_matches('"');
        } else if let Some(stripped) = line.strip_prefix("TOPIC=") {
            topic = stripped.trim_matches('"');
        }
    }

    let octets: [u8; 4] = {
        let mut parts = host_ip.split('.');
        let mut o = [0u8; 4];
        for item in &mut o {
            *item = parts
                .next()
                .expect("invalid IP in .mqtt.config")
                .parse()
                .expect("invalid IP octet");
        }
        o
    };

    (
        Ipv4Address::new(octets[0], octets[1], octets[2], octets[3]),
        host_port,
        client_id,
        topic,
    )
}

fn decode_mqtt_message(msg: &MqttMessage) -> Option<Notification> {
    let mut parts = msg.data[..msg.len].split(|&b| b == b':');
    let service_bytes = parts.next().unwrap_or(&[]);
    let count_bytes = parts.next().unwrap_or(&[]);
    let count = core::str::from_utf8(count_bytes)
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);

    let service = match service_bytes {
        b"slack" => Some(ServiceType::Slack),
        b"gmail" => Some(ServiceType::Gmail),
        b"gcal" => Some(ServiceType::GoogleCalendar),
        _ => None,
    };

    service.map(|s| Notification { service: s, count })
}
