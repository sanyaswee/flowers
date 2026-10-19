# Flowers
DIY scalable plant monitoring & watering system

## About
This is a simple smart-home-ish project about health monitoring and auto-watering the home plants.
The system is scalable and is based on "nodes". Each node is a chip that controls a group of plants.
In theory, the system doesn't care about the number of nodes, the limit comes from Wi-Fi itself, whose theoretical limit is 253 devices.
Unless you grow ~~weed~~ something on an industrial scale, you will never hit it

## Architecture
The system consists of "nodes", each node has several plants (aka channels) connected to it. 
Nodes are managed by a backend server and communicate with it using the [MQTT protocol](https://en.wikipedia.org/wiki/MQTT).
When the node boots, it publishes the boot packet and reports it hardware capabilities. If the node is recognized by the server it responds with a packet that contains its settings, 
if not, the server registers new node with the default settings.
The firmware consists of abstract `core` and the node implementations that are based on it. With that strategy adding a new node version requires just implementing some traits and spawning tasks. 
The backend server also exposes some HTTP endpoints for checking the data and controlling the nodes, intended for building frontend app in the future

## Requirements
To function properly, this project needs a simple home server to be up and running and connected to the same network as the nodes. There is no need for a fancy homelab (tho this project could be a nice part of its infrastructure), an old Android phone could actully run the server.

## Deployment
The project consists of 3 parts, node firmware, backend server, and web frontend. Firmware flashing instructions are located inside the node crate (`firmware/nodes/<node>`). Server deployment instructions are located in the [backend crate](/backend/README.md), local network web app instuctions can be found [here](/web/README.md).

## Current stage & Milestones
For now, this project is in the MVP stage. The major milestones are:
- ~~Build a web frontend with UI for controlling the server and nodes~~
- Introduce ACK / Ping packets in order to track node status
- Add automated watering based on configured threshold / time
- Consider switching to binary transfers instead of JSON packets
- Migrate from SQLite to MySQL or PostgreSQL
- Add another node type (probably ESP32)
- Develop a CLI app for managing the nodes
- Develop a mobile app on the same API endpoints as a web app
- Feature-gate parts of the `core` crate so that different implemetations only get what they need
- Create a virtual node for testing and development purposes
- Dockerization
- Adjustable moisture calibration
- Fix issues
