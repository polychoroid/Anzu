# Runtime Integration: Future Brainstorms

## Purpose and Resume Context

Recorded 2026-10-16. Use this document to resume design discussions after a break. It records context and questions, not approved Rust signatures or implemented features.

Read [Architecture](architecture.md#runtime-rewrite-direction), [API Reference](api.md), and [the contract checklist](../BACKLOG.md#next-step-contract-checklist) first. Exclude everything under `anzu-engine`. Read current source again before implementation because the observations below can become outdated.

Agreed direction:

- `SimulationIO` separates `Command` and `Update` definitions from world/session storage.
- Compute proposes updates through a 1:1 ordered queue. The authoritative state owner commits before publishing permitted events to 1:n session channels.
- Session owns input adapter, extraction-only display adapter, and actor-partitioned permitted state. App owns the renderer and platform lifecycle.
- `session_id` selects a separate delivery channel. `actor_id` partitions data and queries inside that session. Both contribute to defense in depth.
- Local and remote connections share command/update semantics. A remote client does not need a simulation implementation.
- Pub/sub is an analogy for responsibilities, not a requirement for microservices or a broker.

Prediction is not a current priority. Do not require predictive state, rollback, input replay, or a local simulation in remote clients. Keep confirmed state and presentation responsibilities distinct so prediction can be discussed later without promising compatibility with an undefined prediction design.

## Timing: What Independence Actually Means

### Current Code Versus Target Design

The previous summary said that the rendering sequence no longer advances authority. That described a documentation change, not a source change. No runtime behavior was changed.

Current [Session::tick](../anzu-runtime/src/session.rs) passes `Instant::now()` to the server. Current [LocalServer::update](../anzu-runtime/src/server.rs) applies queued commands, calls `simulation.step()` exactly once, and records the timestamp. It does not calculate how many fixed steps are due from elapsed time.

The [winit app](../anzu-winit/src/lib.rs) does not yet run the event loop. Therefore redraw-driven stepping is a risk in the proposed wiring, not an observed running behavior.

### Three Different Responsibilities

1. Authority scheduling decides how much simulation time advances and which commands a step consumes.
2. Session processing applies committed updates that are available locally or from the network.
3. Rendering selects the current permitted view and performs extraction, frame acquire, resource preparation, encoding, submission, and present.

The first produces world changes. The second reconstructs permitted state. The third displays that state. None implies client-side prediction.

### One Step Per Redraw Is Not A Fixed-Time Scheduler

Illustrative example, not a selected tick rate: suppose one step represents 1/60 second of simulation time.

| Redraw rate | One step per redraw | Due steps from elapsed time |
|---|---|---|
| 30 per second | About 0.5 seconds simulated per real second | Usually two steps per callback |
| 60 per second | About one second simulated per real second | Usually one step per callback |
| 120 per second | About two seconds simulated per real second | Often zero steps, sometimes one |

These estimates assume regular callbacks and no catch-up cap. An elapsed-time accumulator determines due fixed steps. A scheduler must also define limits and what happens when it cannot keep up.

### Shared Callback Is Allowed

Logical independence does not require separate threads, services, or timers. A local browser app can use one callback to:

1. Measure elapsed time and service due authority steps, within the selected catch-up policy.
2. Deliver and apply committed session updates.
3. Extract display data and attempt rendering.

The number of simulation steps must not equal the number of redraws by definition. A failed surface acquire must not prevent already-due authority work merely because rendering failed. A failed frame does not roll back a committed batch.

Callback availability still matters. A local scheduler cannot execute while its browser context is suspended. Do not claim uninterrupted wall-clock simulation from an accumulator alone.

### Remote Authority Is Different

A remote host owns its schedule. No session redraw or `send_session_tick()` call authorizes advancement of shared world time. Otherwise clients with different refresh rates or multiple sessions would influence the authority's step rate.

The remote session only sends intent and processes committed results. It may display unchanged confirmed state across several frames. That is an acceptable baseline before smoothing or prediction.

### Pause and Recovery Are Still Open

Choose local hidden-tab behavior explicitly: pause simulation time, discard some elapsed time, or perform bounded catch-up. A remote host can continue while a client is hidden; that client then needs update catch-up or a fresh snapshot. Local and remote pause policies do not have to be identical.

Verify timing with controlled elapsed-time inputs and callback rates, skipped frames, render failures, and pause/resume. Do not use an actual monitor refresh rate as the only scheduler test.

## Topic 1: IO and Connection Contracts

Why first: `SimulationIO` exists, but `Simulation` still has a separate `Command`. `Session<Input, Sim>` and `Server<S>` still depend on `Simulation`.

Questions:
- Does `Simulation` select an associated `IO: SimulationIO`?
- What associated IO and operations does the connection trait expose? The platform host selects a concrete local or remote type; no runtime selection enum is needed.
- Where do snapshots, status, rejections, and version metadata belong relative to `Command` and `Update`?
- What does nonblocking `dequeue_update()` return for no data, failure, closure, and a complete batch?
- What happens to a command when queue acceptance fails?

Bring: [simulation](../anzu-runtime/src/simulation.rs), [server](../anzu-runtime/src/server.rs), and [session](../anzu-runtime/src/session.rs).

Desired outcome: agreed associated-type relationships and error semantics, with local and remote type sketches that do not require a remote simulation instance. Track Feature 12.6 and the contract portion of Feature 12.4.

Decision update, 2026-10-17: local versus remote is hardcoded by the platform host. Remove runtime enum selection from future discussion scope. The current source enum is implementation to replace, not a target requirement. Generic session code should work with either concrete connection without requiring dynamic dispatch.

## Topic 2: Input Intent and Actor Identity

Why: input normalization currently returns `InputSignal`, while `simulation.signal_command()` maps that signal using simulation state. A remote client does not own that simulation.

Questions:
- Does domain mapping use permitted session context, or occur on the authority after receiving a signal?
- Which context identifies the submitting session and actor, and how is membership checked?
- How are unsupported controls, held input, and commands queued between steps handled?

Desired outcome: an input-to-command contract compatible with local and remote use, without unsupported-input panics or trusting payload IDs as authorization. Track Features 12.6 and 17.5.

## Topic 3: Compute Reads and Update Application

Why: compute backend choice should not require replacing state-application semantics.

Questions:
- How does compute obtain a stable state revision? Start with one batch in flight, or permit concurrency?
- Are updates final values, operations, or a limited combination?
- How are stale proposals rejected or recomputed, and when is a batch considered committed?
- Can authoritative and permitted session state apply the same updates, or is a filtered projection required?
- How is committed publication retained or recovered if delivery fails? Persistence is not yet required.

Desired outcome: compute input/output and apply contracts, including batch ordering, failure behavior, and a small reconstruction example. Do not assume CPU/GPU numerical parity. Track Feature 12.7.

## Topic 4: Session Storage and Disclosure

Why: session state and actor-scoped queries do not yet exist.

Questions:
- What permitted state is stored for each actor, and how is shared data represented?
- How do query interfaces exclude other actor partitions?
- How are membership changes, permission revocation, and entity entry/removal represented?
- Can extraction observe a partially applied batch, or only stable state between batches?

Desired outcome: session storage and query contracts, disclosure rules, and negative tests for wrong-session delivery and wrong-actor reads/commands. Track Features 12.8 and 17.5.

## Topic 5: Snapshots and Stream Continuity

Why: incremental updates need an initial state and a recovery path.

Questions:
- How is a permitted snapshot tied to the next subscription batch without a gap?
- Which world revision and subscription sequence values are needed?
- How are duplicates, missing batches, retries, disconnects, and queue saturation handled?
- When does a slow session request a fresh snapshot instead of replaying updates?

Desired outcome: initialization/recovery state transitions and bounded-queue rules. Track Features 12.8 and 12.4.

## Topic 6: Scheduling and Browser Lifecycle

### Latest Clarification and Design Intent

The earlier simulation/redraw explanation covers only part of scheduling. The runtime has several activities with different, possibly dynamic frequencies and different tolerances for lost data and missed deadlines. Do not reduce this design to one simulation loop inside one rendering loop.

The user chose Rust to support predictable, tunable execution. The goal is precise control with measured error bars across platform hosts, not perfect timing or identical host behavior. Interpret strong wording as an engineering direction unless a guarantee is explicitly requested. Raise platform constraints when they change a design decision; do not repeatedly qualify goals the user already understands.

Distinguish requested cadence from achieved cadence. Measure service intervals, lateness, queue age, and missed deadlines. Preserve activity semantics across hosts while adapting clocks, wakeups, and execution facilities to each host. Rust provides useful control over allocation and ownership; host scheduling and device timing still need measurement.

### Activity Inventory

These are discussion starting points, not selected rates or loss policies.

| Activity | Possible trigger | Timing or delivery question |
|---|---|---|
| Input capture | Platform event | Which transitions must be preserved in order? |
| Input polling | Sampling deadline | Which samples can be replaced without losing short transitions? |
| Command enqueue and send | Intent arrival or batching deadline | Which discrete actions must survive, and which continuous intent can be combined? |
| Authority computation | Simulation deadline or available work | What fixed-time, lateness, and catch-up policies apply? |
| Authoritative commit and publication | Completed proposal | How are ordering and accepted changes preserved? |
| Update receive | Transport event | How are buffering, disconnects, and capacity bounded? |
| Update dequeue and apply | Available batches and work budget | Which dependencies require complete ordered application? |
| Display extraction | State/view change or display demand | Can obsolete intermediate views be skipped? |
| Draw request and rendering | Display opportunity or invalidation | When should a pending request be combined or a frame skipped? |
| Audio production and events | Buffer demand and timed events | What prevents underruns, and what happens to late events? |
| Asset loading and GPU upload | Requests, completions, and budgets | How is work prioritized without blocking critical activities? |
| Connection maintenance | Heartbeat, timeout, or reconnect deadline | What must progress independently of rendering? |
| Diagnostics | Counters, errors, and sampled reporting | Which samples can be lost, and which failures need retained evidence? |

Frequency and loss tolerance are separate properties. An infrequent consumer can still require every item. Skipping extraction does not permit dropping dependent updates. Axis state, a fire-button press, and a text character need not share a delivery policy.

### Per-Activity Contract

Define trigger, fixed/adaptive rate policy, deadline, ordering dependencies, work budget, and overload behavior for each activity. Possible overload behaviors include bounded queueing, combining, replacing, skipping, rejecting, or resynchronizing. Decide which are valid for each data type rather than selecting one global policy.

Some triggers come from the browser, transport, display, or audio system. Separate cadences do not automatically require separate threads or timers. Do not turn `Session::tick()` into an undefined do-everything operation, or build a general-purpose scheduler before these responsibilities are understood.

Use the timing explanation above as context, not as a selected scheduler implementation. Scheduling correctness is not prediction.

Questions:
- Which activities are event-driven, periodic, adaptive, or externally demand-driven?
- Which items must be preserved individually, and which consumers need only the latest state?
- How can each rate change, and what happens to pending work when it does?
- What deadlines, budgets, and overload policies apply to each activity?
- Which host timing capabilities are needed, and how will achieved behavior and variation be measured?
- Who owns elapsed-time accumulation and the authority scheduler in local mode and on the remote host?
- What tick duration, catch-up cap, and hidden-tab policy are appropriate for the first slice?
- How is command consumption ordered relative to fixed ticks?
- Which callback services local steps, session updates, and redraw requests?

Desired outcome: an activity-contract matrix before choosing rates or execution placement. Include no one-step-per-redraw assumption, no client-driven remote authority, explicit suspension policy, and measured timing variation. Track Features 12.7 and 12.9 with Features 1.1, 12.5, and Initiative 14. Audio requirements belong in the inventory even if audio implementation remains later work.

Suggested restart: classify the first slice's input, commands, committed updates, extraction, drawing, and audio requirements. Resolve data preservation before choosing a collection of timers.

## Topic 7: Display Extraction and Web Integration

Questions:
- Which stable view of session-permitted state does the display adapter read?
- How does display composition select views or cameras when the session contains multiple permitted actors?
- How are sessions, renderer surfaces, and physical displays mapped? For example, can two local single-actor sessions present to two monitors?
- What minimal backend-independent render data and resource handles are needed for the first visible result?
- How are initialization, resize, surface failure, and missing assets handled?
- How does App coordinate session and renderer without taking ownership of simulation rules?

Actor scope is a boundary between authoritative world state and each session's permitted state. It does not restrict a shared display to one actor. A display adapter can compose multiple actors already permitted to its session. Separate sessions can also be bound to separate renderer surfaces and monitors.

Desired outcome: display trait and render-data contracts, display/session topology, plus a browser acceptance checklist. No prediction is required. Track Feature 12.9 with Feature 2.7.

## Topic 8: Remote Host and Transport

Questions:
- What is the host package boundary and name?
- Which codec, versioning policy, transport library, and size/rate limits fit the first deployment?
- Does WebSocket over TLS satisfy the measured requirements, or is there a demonstrated need for WebTransport?
- How are authenticated sessions bound to channels and command acknowledgements across reconnect?

Desired outcome: a transport/host contract using the established local semantics, plus a local/remote parity test plan. No broker or microservice deployment is implied. Track Feature 12.10.

## Later Consideration: Prediction and Smoothing

Revisit only after contracts, committed-update reconstruction, browser output, and remote recovery are working and validated. First measure whether response latency or presentation quality warrants more complexity.

Prediction estimates state before authority confirmation. Interpolation displays between known states. They are different techniques, and neither is needed to establish scheduling correctness.

Future questions include acceptable presentation delay, correction behavior, and whether any client-side computation is justified. Do not design rollback buffers or replay systems during the current contract work. Track Feature 12.3 as deferred consideration, not the next implementation task.

## Topic 9: Domain Models, Scale, and Propagation

This topic records a separate simulation-modeling brainstorm. It is vocabulary and design context, not an approved runtime contract. Domain examples are illustrative and do not select a numerical method or CPU/GPU placement.

### Core Vocabulary

| Term | Meaning in this discussion |
|---|---|
| Simulation | An executable model of a domain. It defines represented state, rules that change state, and observable results. CPU or GPU execution does not define the domain model. |
| Domain model | The state, object types, actions, rules, assumptions, and outputs for one modeled domain. Each domain can have its own type universe. |
| Multi-domain simulation | Several domain models that exchange explicit results while retaining their own state and rules. |
| Scale | A level of detail, such as particles, individuals, groups, or settlement-wide quantities. Micro and macro name relative scales, not fixed universal levels. |
| Aggregation | Reduction of detailed state into summaries. A summary can be derived from individuals, or a coarse model can evolve aggregate state directly. Aggregation can discard information. |
| Multiscale model | A model that represents or relates more than one scale and defines how information passes between them. |
| Simulation LOD | A runtime choice to use different model detail or update cost based on relevance. It changes how behavior is computed. Rendering LOD changes visual detail instead. |
| Propagation | Model-defined evolution or transport of an influence through time and possibly space or a network. The domain defines the mechanism, timing, and effect at reception or interaction. |

These terms describe different axes. Scale is not the same as aggregation. A model can retain individual state and calculate aggregate summaries. It can also simulate coarse aggregate state without representing individuals. A multiscale model connects levels. Simulation LOD selects among model details during execution.

Each domain owns its object, action, state, and rule types. Shared infrastructure may schedule models and carry explicitly typed results between them. It should not require one universal object or action vocabulary.

### Multi-Domain and Hierarchical Examples

A settlement model could combine dwelling environment, people and society, economy, and settlement development. Room conditions can affect occupants. Occupants create demand for goods and labor. Trade affects available materials and construction. Each exchange should state what information crosses the domain boundary.

An RTS can retain individual unit health, ammunition, damage type, and position where these details affect outcomes. Unit state can produce battalion summaries, then army and campaign summaries. Alternatively, a higher level can use its own coarse rules. Totals must preserve distinctions that affect outcomes, such as ammunition type or force composition. A coarse representation cannot answer questions about details it discarded.

Simulation LOD can use detailed unit behavior near active play and cheaper formation or campaign rules elsewhere. Distance can help select detail, but relevance and interaction can also matter. When changing representation, define how retained information is summarized and how detailed state is reconstructed.

Command and report flow can form coupled observe-orient-decide-act cycles at multiple hierarchy levels. Reports flow upward and orders flow downward through command links or communication systems. Radios, couriers, and relays can have different delay, range, capacity, reliability, and routing rules. Each level decides from information it has received, not from assumed perfect access to current world state.

### Propagation Examples

Propagation is a first-class domain concept, but it is not one universal mechanism. Instant influence can be an explicit model assumption where its delay is irrelevant. Other models can represent local interactions, evolving fields, scheduled message delivery, or motion through space. These mechanisms should not share domain payload types merely because they involve propagation.

- In a massless ideal spring, force transmission is treated as instant. A real elastic spring has distributed mass and finite-speed stress waves. A model that needs this behavior must represent distributed material state or a segmented approximation, with spatial and temporal resolution that captures the wave.
- In a spark-ignition engine, heat, reactive species, flame growth, pressure, and motion are coupled processes with different mechanisms and rates. A field model can approximate these processes without tracking every molecule.
- A projectile is a propagating entity. Its trajectory can be calculated from projectile and environment parameters or sampled from ballistic tables. Interpolation estimates values between samples. Interaction checks must cover the swept path so a fast projectile cannot skip over a target between endpoints.
- A radio order is a propagating message. Its arrival is governed by a communication model. A projectile is instead a moving modeled object, and an elastic disturbance can emerge from local material equations. Keep these domain mechanisms distinct.

The transactional interpretation of quantum mechanics was raised as a reminder that not every interpretation treats interactions as ordinary local, time-delayed transport. It is a distinct interpretation of quantum theory, not a default mechanism for engineering models or a reason to encode all interactions as one propagation type.

### Open Questions

- Which domains need individual, aggregate, and multiscale representations in the first useful simulation profile?
- Which cross-domain exchanges are values, events, commands, or evolving signals, and what timing guarantees apply to each?
- Which propagation behaviors need explicit spatial travel, and which can use a declared zero-delay approximation?
- What data must survive when simulation LOD changes representation?
- Which physical models need a numerical field or solver, and which can use calibrated aggregate rules?

Desired outcome: agree on domain-specific terms and exchange boundaries before proposing shared runtime traits or a general propagation API. Validate each model against the questions it must answer, including its stated assumptions and acceptable error.