# Receive and Display Web Notifications with ESP32

This project only supports the ESP32 with Xtensa architecture due to the strict dependency on the `esp-hal` crate.

## Setup

1. Refer [impl Rust for ESP32](https://esp32.implrust.com/dev-env.html) to install toolchains and set up the environment.

1. Prepare `.wifi.config` file in the project root with the following content:

    ```txt
    SSID="your_wifi_ssid"
    PASSWORD="your_wifi_password"
    ```

1. Prepare `.mqtt.config` file in the project root with the following content:

    ```txt
    HOST_IP_ADDRESS="your_mqtt_broker_ip_address"
    HOST_PORT="your_mqtt_broker_port"
    CLIENT_ID="your_mqtt_client_id"
    PORT="your_mqtt_port"
    ```

1. Start the MQTT broker. If you have `mosquitto` installed, you can run the following command:

    ```bash
    mosquitto -p {your_mqtt_broker_port} -c broker/mosquitto.conf
    ```

1. Install `espflash` and `justfile` to call commands in `justfile`.  
    Build, flash and monitor the device with:

    ```bash
    just flash
    ```
