import paho.mqtt.client as mqtt
import sys

def get_unread_counts():
    return {"gmail": 2, "slack": 1}

def load_mqtt_config():
    config = {}
    with open(".mqtt.config", "r") as f:
        for line in f:
            key, value = line.strip().split("=", 1)
            config[key] = value.strip('"')
    return config

if __name__ == "__main__":
    
    config = load_mqtt_config()
    topic = config.get("TOPIC", "shirasesp/notify")
    mqtt_port = config.get("MQTT_PORT", "1883")

    try:
        # subprocess.run(["mosquitto", "-p", mqtt_port, "-d"], check=True)
        client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
        client.connect("localhost", int(mqtt_port))

        counts = get_unread_counts()
        for service, count in counts.items():
            message = f"{service}:{count}"
            client.publish(topic, message)
    
    except KeyboardInterrupt as _:
        sys.exit(0)