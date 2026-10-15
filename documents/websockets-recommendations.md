# Multiplayer WASM + WebSocket Architecture Recommendations

## Current Project Status Note

Anzu does not currently provide a production multiplayer or WebSocket service. This document gives design recommendations. It does not describe implemented behavior.

## Overview

This document outlines one possible architecture for a multiplayer application that uses:

* **WASM clients (browser-hosted)**
* **WebGPU for rendering**
* **WebSockets for networking**
* **Authoritative server model**

The design goals are:

* Low perceived latency
* Scalable networking
* Efficient bandwidth usage
* Smooth gameplay across varying network conditions

***

# Core Principles

## 1. Authoritative Server Model

In a server-authoritative design, the server owns shared world state:

* Clients send **input**
* Server validates and applies game logic
* Server distributes the resulting state

This design limits the authority of clients. It does not prevent every form of cheating or guarantee identical client views. [Source](https://generalistprogrammer.com/tutorials/game-networking-fundamentals-complete-multiplayer-guide-2025)

***

## 2. Client-Side Prediction

Client-side prediction can update local presentation before the server responds.

One approach is:

1. Player generates input
2. Client simulates immediately
3. Input is sent to server asynchronously

Prediction can reduce perceived input delay. It does not remove network latency, and the server can correct the predicted state. [Source](https://danieljimenezmorales.github.io/2025-06-20-client-side-prediction-and-server-reconciliation/)

***

## 3. Server Reconciliation

Because the server is authoritative:

* The client’s predicted state may diverge
* The client must periodically correct itself

Typical approach:

1. Receive authoritative state from server
2. Rewind to last confirmed tick
3. Replay unacknowledged inputs

This can bring the client view back in line with authoritative state. The result depends on prediction and correction policy. [Source](https://zacksinisi.com/multiplayer-client-side-prediction-and-server-reconciliation-demystified/)

***

## 4. Interpolation for Remote Entities

For objects not controlled by the client:

* Render them slightly **in the past**
* Interpolate between known states

This can smooth remote motion when the client has enough recent snapshots:

```
[t0] ------- [t1]
     render here
```

Interpolation adds presentation delay and does not remove packet loss or server-side state changes. [Source](https://github.com/0xFA11/MultiplayerNetworkingResources)

***

# Networking Model

## Client → Server

Send domain commands or input intent. Do not accept client-authored world state as authoritative:

```json
{
  "tick": 10234,
  "input": {
    "forward": true,
    "turn": -0.3
  }
}
```

Possible benefits:

* Minimal bandwidth
* Deterministic simulation
* Enables prediction

***

## Server → Client

Send **state snapshots or deltas**:

```json
{
  "tick": 10240,
  "entities": [...]
}
```

Optimizations:

* Delta compression
* Interest filtering
* Quantization

***

# Frame Execution Model

## Client Loop (WASM)

Per frame:

```
1. Read input
2. Apply prediction (local simulation)
3. Send input to server
4. Receive updates
5. Reconcile if needed
6. Interpolate remote entities
7. Render via WebGPU
```

***

## Server Loop

Per tick:

```
1. Receive client inputs
2. Validate + simulate world
3. Build per-client state
4. Send filtered updates
```

***

# WebSocket Architecture

## Key Properties

* Full-duplex, persistent TCP connection
* Low-latency communication
* Widely used for multiplayer games [\[w3tutorials.net\]](https://www.w3tutorials.net/blog/nodejs-websocket-performance/)

***

## Concurrency Model

Do not select a concurrency model without measuring the expected workload.

For many concurrent connections, consider asynchronous or event-driven I/O. Compare it with other models under representative load.

Examples include Node.js event loops, Rust Tokio, and Go goroutines. The suitable model depends on workload, runtime, and operational constraints. [Background](https://codingtechroom.com/question/multithreading-websockets)

***

## Scaling Considerations

Each WebSocket connection uses memory and keeps connection state. The amount depends on the server and protocol. One source estimates approximately 20-50 KB per connection. Treat this as a rough estimate, not a capacity guarantee.

Choose load balancing, multiple servers, and resource limits from measured capacity needs. [Background](https://dev.to/young_gao/scaling-websocket-connections-from-single-server-to-distributed-architecture-1men)

***

# Bandwidth Guidelines

## Typical Per-Client Usage

| Direction          | Example estimate |
| ------------------ | ---------------- |
| Upstream (input)   | 1-10 KB/s        |
| Downstream (state) | 5-50 KB/s        |
| Total              | about 10-60 KB/s |

These values are rough planning examples, not measured Anzu traffic. Measure serialized traffic at the expected update rate and player count before setting limits.

***

## Key Observations

* Measure bandwidth, latency, serialization cost, and server processing time.
* Set per-connection and aggregate traffic limits in the application protocol.
* Do not assume that WebSocket transport provides an application bandwidth budget.

***

# Optimization Strategies

## 1. Send Commands, Not Client State

Prefer a domain command such as:

```
"move forward"
```

Do not accept a client-authored authoritative state change such as:

```
"set position = (x,y)"
```

***

## 2. Select a Message Format

Compare JSON and binary formats with representative messages. Measure payload size, encode/decode cost, tooling needs, and compatibility requirements.

Possible binary formats include:

* FlatBuffers
* Protobuf
* Custom packed structs

***

## 3. Consider Delta Encoding

Send deltas when they reduce traffic and the receiver can detect missing or out-of-order state. For example:

```
position += delta
```

***

## 4. Quantization

Example:

```
float32 → uint16
```

Quantization can reduce payload size. Define acceptable error bounds and test the effect on simulation and presentation.

***

## 5. Message Aggregation

Consider aggregation when it reduces overhead without adding unacceptable latency:

```
1 packet / tick
```

Avoid excessive small messages when they add overhead without improving latency:

```
many tiny packets
```

***

# Large World Handling

## Scope

Some clients do not need a complete copy of a large world. Define the visible state for each session.

***

## Solutions

### 1. Interest Management

Send only the entities relevant to each session, such as:

* Nearby players
* Visible objects

***

### 2. Spatial Partitioning

Choose a spatial structure that fits the world and query pattern, such as:

* grids
* quadtrees
* chunks

***

### 3. World Streaming

Load and unload world regions when the product requires streaming:

```
[Chunk A][Chunk B][Chunk C]
         ↑ player
```

***

# Advanced Techniques

## Lag Compensation

Lag compensation can evaluate an action against recent historical state. Add it only when the application needs this behavior, and define its time and trust limits.

***

## Hybrid Authority Models

An application can assign different authority by action:

* client authority for low-risk actions
* server authority for critical logic

***

## Rollback Networking (optional)

Rollback can suit deterministic simulations that need to replay recent inputs:

* rewind entire state
* replay inputs

This approach requires strict determinism, state history, and replay tests.

***

# Architecture Summary

## Data Flow

```
Client:
  Input → Predict → Send
                  ↓
Server:
  Validate → Simulate → Broadcast
                  ↓
Client:
  Reconcile → Interpolate → Render
```

***

## Design Checks

* Keep world authority on the server when the design uses a server-authoritative model.
* Define how the client predicts and corrects local state.
* Limit messages by size, rate, and session authorization.
* Send each session only the state it is allowed to see.
* Measure latency, traffic, serialization cost, and server work under representative load.

***

# Summary

* Client prediction can reduce perceived input delay, but it cannot remove network latency.
* Server authority can limit client control of shared state, but it does not guarantee security by itself.
* Measure traffic and processing costs before selecting update rates or scaling strategies.
* Choose WebSocket and concurrency designs from protocol needs and load tests.
* Treat this guide as a proposal until Anzu implements and validates a network service.

***

# References

* [Multiplayer networking techniques](https://github.com/0xFA11/MultiplayerNetworkingResources)
* [Client-side prediction and latency](https://danieljimenezmorales.github.io/2025-06-20-client-side-prediction-and-server-reconciliation/)
* [Server-authoritative architecture](https://generalistprogrammer.com/tutorials/game-networking-fundamentals-complete-multiplayer-guide-2025)
* [Prediction and reconciliation](https://zacksinisi.com/multiplayer-client-side-prediction-and-server-reconciliation-demystified/)
* [WebSocket performance](https://www.w3tutorials.net/blog/nodejs-websocket-performance/)
* [Concurrency and WebSocket servers](https://codingtechroom.com/question/multithreading-websockets)
* [WebSocket scaling](https://dev.to/young_gao/scaling-websocket-connections-from-single-server-to-distributed-architecture-1men)

***
