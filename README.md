# Flowers
DIY scalable plant monitoring & watering system

## About
This is a simple smart-home-ish project about health monitoring and auto-watering the home plants.
The system is scalable and is based on "nodes". Each node is a chip that controls a group of plants.
In theory, the system doesn't care about the number of nodes, the limit comes from Wi-Fi itself, whose theoretical limit is 253 devices.
Unless you grow ~~weed~~ something on an industrial scale, you will never hit it

## Current stage & Milestones
For now, this project is in the MVP stage. The major milestones are:
1. Build a web frontend with UI for controlling the server and nodes
2. Introduce ACK packets for settings overrides and other types
3. Add automated watering based on configured threshold
4. Consider switching to binary transfers instead of JSON packets
5. Migrate from SQLite to MySQL or PostgreSQL
6. Add another node type (probably ESP32)
7. Develop a CLI app for managing the nodes
8. Develop a mobile app on the same API endpoints as a web app
9. Fix all current issues

## Requirements
To function properly, this project needs a simple home server to be up and running and connected to the same network as the nodes. 
Some functionality may work without it, but configuring nodes requires action from the server

## Setup instructions
For the server, you just need to [install some dependencies](/backend/README.md) and `cargo run` the `backend` directory on the host machine, and for the firmware you'll find the instructions for a particular node type in `firmware/nodes/<your_node>`
