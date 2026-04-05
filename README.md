# Receive and Display Web Notifications with ESP32

This project only supports the ESP32 with Xtensa architecture due to the strict dependency on the `esp-hal` crate.

## Setup

1. Refer [impl Rust for ESP32](https://esp32.implrust.com/dev-env.html) to install toolchains and set up the environment.

1. Prepare `.wifi.config` file in the project root with the following content:

    ```txt
    SSID="your_wifi_ssid"
    PASSWORD="your_wifi_password"
    ```

1. Install `espflash` and `justfile` to call commands in `justfile`.  
    Build, flash and monitor the device with:

    ```bash
    just flash
    ```
