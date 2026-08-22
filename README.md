# Flowers
DIY scalable plant monitoring & watering system

## About
This is a simple smart-home-ish project about health-monitoring and auto-watering the home plants.
The system is scalable, and is based on "nodes". Each node is a chip that controls a group of plants.
In theory, the system doesn't care about the number nodes, the limit comes from Wi-Fi itself, whose theoretical limit is 253 devices.
Unless you grow ~~weed~~ something on industrial scale, you will never hit it

## Current stage & Milestones
For now, this project is in MVP stage. The big milestone are;
1. Build a web frontend with UI for controlling the server and nodes
2. Add another node type (probably ESP32)
3. Fix all current issues

## Requirements
To function properly, this project needs and simple home server to be up and running and connected to the same network as nodes. 
The auto-watering could work without it, but configuring nodes requires the action from the server

## Setup instructions
For the server, you just need to `cargo run` the `backend` directory on the host machine, and for the firmware you'll find the instructions for a particular node types in `firmware/nodes/<your_node>`