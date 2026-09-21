# Backend Server
The server for handling data and controlling the nodes.
It exposes HTTP endpoints used for interaction with the nodes. The documentation is available on [localhost:3000/docs](localhost:3000/docs)

## Requirements
You should have the MQTT broker like `mosquitto` to be running

### Debian/Ubuntu
```
sudo apt update
sudo apt install mosquitto-clients
```
Then you should update the config file:
```
sudo nano /etc/mosquitto/conf.d/local.conf
```
Add these lines:
```
listener 1883 0.0.0.0
allow_anonymous true
```
And finally:
```
sudo systemctl restart mosquitto
sudo ufw allow 1883
```

## Running
After all requirements have been satisfied:
```
cargo run --release
```