# Anzu Roadmap

## Personas

- **Developer:** Builds the runtime and ROM packages, prepares content, and validates behavior.
- **User:** Selects and runs ROMs. A user can play a game or conduct a physics experiment.
- **Session Host:** Starts a shared session, assigns permitted actors, and maintains the session during local or remote play.

## Product Goal

Anzu is a browser-first simulation and visualization system. A user can select a ROM, see it run, control its state, switch to another ROM, and play with other users. A developer can define ROM content and parameters as validated data while reusable runtime behavior remains in code. A user can run repeatable physics benchmarks and inspect timing and workload results.

## Roadmap Rules

- Order work by the demonstrable user outcome, not by subsystem or legacy feature number.
- Every milestone has at least one Gherkin scenario with observable results.
- Mark a milestone complete only when its scenarios pass and its stated evidence exists.
- Treat unverified work as planned. Do not infer completion from old notes, abandoned code, or a successful build alone.
- Keep implementation contracts and detailed API choices in the architecture and API documents. This roadmap records user outcomes, dependencies, and acceptance.
- Keep `anzu-engine` out of this roadmap's implementation scope. Use `anzu-runtime`, `anzu-winit`, `anzu-render-wgpu`, and the active ROM packages.

## Data-Driven Architecture

ROM packages provide versioned, validated data for catalog entries, scenes, assets, control mappings, and tunable scenario parameters. The runtime loads this data and connects it to reusable simulation and rendering behavior implemented in code. The first roadmap stage does not require user-authored scripts or rules. Data layout choices, such as array-of-structures or structure-of-arrays, are separate decisions and require workload evidence.

## Journey 1: Open Anzu and See a ROM

### Milestone 1: Open the browser app

**Demonstrable result:** The browser displays an initialized application window and a ROM catalog. Initialization failure has a visible error state and does not leave a blank, unusable page.

**BDD scenario: Browser startup**

```gherkin
Feature: Browser startup
  Scenario: Open the application
    Given the user opens the Anzu browser app
    When the app initializes
    Then the application window is visible
    And the ROM catalog is available
    And an initialization failure is shown as an actionable error
```

**Depends on:** Browser platform shell, canvas lifecycle, and application readiness state.

### Milestone 2: Select a ROM and draw a shape

**Demonstrable result:** The user selects the built-in `Triangle Man` demo from the catalog and sees its shape on screen. The initial scene and asset references come from validated, versioned ROM data.

**BDD scenario: Launch the demo ROM**

```gherkin
Feature: Select and launch a ROM
  Scenario: Draw the selected demo
    Given the ROM catalog contains the Triangle Man demo
    When the user selects and launches that ROM
    Then the demo shape is visible in the browser window
    And the loaded scene matches the validated ROM data
    And a load failure identifies the invalid or missing data
```

**Depends on:** ROM catalog selection, versioned scene data, asset resolution, render extraction, and a browser renderer that can acquire, encode, submit, and present a frame.

## Journey 2: Play and Control a ROM

### Milestone 3: Change object state with user control

**Demonstrable result:** The user launches `You Touched My Boats` and uses keyboard controls to change a boat's authoritative state. The visible boat state follows committed updates, not uncommitted computation proposals.

**BDD scenario: Control a boat**

```gherkin
Feature: Control a boat
  Scenario: Apply a keyboard command
    Given the user has launched You Touched My Boats
    And a boat is assigned to the user's actor
    When the user presses a mapped movement key
    Then the input maps to a domain command without a panic
    And the authority computes and commits the resulting update
    And the user's permitted session state applies the committed update
    And the next display extraction shows the changed boat state
```

**Initial input scope:** Keyboard controls. ROM control mappings and tunable parameters are data. The ROM's reusable simulation rules remain code.

**Depends on:** Command and update contracts, input-to-command mapping, stable simulation reads, authoritative commit, session state, permitted update delivery, display extraction, and elapsed-time authority scheduling independent of redraw frequency.

## Journey 3: Choose a Different ROM

### Milestone 4: Stop one ROM and start another with fresh state

**Demonstrable result:** The user exits the active ROM, selects another catalog entry, and starts that ROM from its initial state. The first catalog contains playable `You Touched My Boats` and a selectable `Triangle Man` demo.

**BDD scenario: Switch ROMs**

```gherkin
Feature: Switch ROMs
  Scenario: Start a different ROM with fresh state
    Given the user is playing You Touched My Boats
    When the user exits that ROM and selects the Triangle Man demo
    Then the boat simulation is stopped
    And the Triangle Man demo starts from its initial state
    And no state from the previous ROM appears in the new session
```

**Depends on:** ROM lifecycle, catalog entries, session teardown, and clean initialization from the selected ROM's validated data.

## Journey 4: Play Together on One Display

### Milestone 5: Support local two-player play

**Demonstrable result:** Two users control separate boats in one local session on one shared display. The first input layout uses separate keyboard key groups. Both users see committed state permitted to that session.

**BDD scenario: Play locally together**

```gherkin
Feature: Local multiplayer
  Scenario: Two users control separate boats
    Given a local You Touched My Boats session is active
    And each user is assigned a distinct authorized actor
    When both users send commands through their keyboard key groups
    Then each command is applied to its assigned actor
    And the shared display shows both boats from committed session state
    And a command for the wrong actor is rejected
```

**Depends on:** Actor identity, local connection contract, session and actor partitions, command authorization, complete ordered update batches, and multi-actor display composition.

## Journey 5: Play Together Remotely

### Milestone 6: Join a hosted remote session

**Demonstrable result:** A host runs authority and two authenticated remote users join a shared session. The host validates actor permissions. A disconnected user can reconnect and recover a permitted state without a snapshot/update gap.

**BDD scenario: Join and recover a remote session**

```gherkin
Feature: Remote multiplayer
  Scenario: Join, play, disconnect, and recover
    Given a host has started an authenticated game session
    And each remote user is bound to an authorized actor
    When a user sends a valid command
    Then the host commits the update before publishing it
    And each session receives only its permitted state
    When that user disconnects and reconnects
    Then the session resumes from a permitted snapshot and ordered updates
    And duplicate, missing, or unauthorized data is rejected or recovered explicitly
```

**Depends on:** Local command/update semantics, host package boundary, authenticated session binding, actor authorization, versioned bounded messages, reliable ordered delivery, bounded queues, and snapshot/reconnect recovery.

**Transport decision:** The transport remains open until the local contracts and workload are measured. WebSocket over TLS is a candidate, not a requirement.

## Journey 6: Run a Repeatable Physics Benchmark

### Milestone 7: Measure a named workload

**Demonstrable result:** A user selects a repeatable physics workload, runs it with recorded settings, and inspects timing distributions and work counts. CPU and GPU timing are reported separately. Unsupported GPU timing is reported as unavailable.

**BDD scenario: Compare repeated benchmark runs**

```gherkin
Feature: Physics benchmarks
  Scenario: Record repeatable timing and workload results
    Given a named physics workload and fixed run settings
    When the user runs the benchmark with warm-up and measured samples
    Then the result identifies the target, backend, workload, and settings
    And it reports p50, p95, and p99 stage timings with work counts
    And it reports CPU and GPU timing separately
    And an unsupported GPU timing source is marked unavailable
    And instrumentation mode and overhead are recorded
```

**Initial research scope:** Benchmark output and workload counts. Model comparison, saved trajectories, interactive inspection, and experiment authoring are not required for this milestone.

**Depends on:** A stable simulation workload, stage boundaries, low-cost health counters, sampled timing, and machine-readable benchmark output.

## Cross-Journey Contracts

These contracts apply wherever the related work appears in the journeys above:

- **Authority:** Computation proposes updates. The authoritative state owner validates and commits them before publication.
- **Session:** A session stores only permitted state. Actor authorization filters data before it enters session state. Display composition may include any actors permitted to that session.
- **Rendering:** The session-owned display adapter reads a stable permitted view and emits backend-independent render data. The renderer owns frame acquire, resource preparation, command encoding, queue submission, and present.
- **Scheduling:** Authority progression does not depend on redraw frequency. Local and remote hosts can use different scheduling facilities, with measured timing error.
- **Delivery:** Required committed updates are not silently dropped. Empty queues, closure, failures, duplicates, gaps, and recovery have distinct outcomes.
- **Data:** ROM data is versioned and validated at the load boundary. Invalid references and unsupported versions produce actionable errors.
- **Performance:** Record target, backend, workload, warm-up, sample count, timing distributions, work counts, instrumentation mode, and overhead. Do not use CPU submission time as GPU time.

## Completion Evidence

For each milestone, retain:

- Passing automated tests for its acceptance scenarios and failure cases.
- Native and browser-target build checks where the milestone changes those targets.
- A manual browser demonstration when the result is visible or interactive.
- Benchmark output for performance claims, with the workload and measurement settings.

A milestone remains planned until the evidence is recorded. Record limitations and measured variation. Do not claim identical timing across hosts.

## Deferred

These items are not part of the committed journey order. Revisit them only when a confirmed user outcome requires them:

- Prediction and interpolation for remote presentation.
- User-authored simulation rules or scripting.
- GPU-resident authority, hybrid physics, and CPU/GPU numerical parity.
- Multi-display local topology.
- Blender GLB import, large-scene streaming, and residency policy.
- Audio, persistence, replay, and editor workflows.
- Physics model comparison and interactive state inspection.
