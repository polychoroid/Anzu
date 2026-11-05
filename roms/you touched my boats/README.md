# You Touched My Boats: Design Notes

This document records the current game-design discussion for the `You Touched My Boats` ROM package. It is a working note, not a complete game specification. Open decisions are listed at the end.

## Implemented Prototype Status

The ROM currently creates two player records and two boat records. Its `step` method increments a tick counter. Input signals map to action values, but `apply` only logs each action. `GameState::apply_update` is unfinished, and the ROM does not add items to its update queue. The current display projection emits one rotating asset instance; it does not display the boat records.

The following sections describe game intent and proposed behavior. They are not implemented simulation features. The active renderer loads a GLTF asset, but it does not yet render the boat data or implement the collision design below.

## Game Concept

The game is a two-player, top-down boat combat game. The desired feel is a tense approach followed by a dangerous open-water fight. Later versions can add land, cover, and defensive positions.

The first scene has two players. Each player controls one boat. The boats start on opposite sides of the play area and face inward. There is no land in the first scene.

## Scale and Combat

The current scale model has three gameplay ranges:

1. **Approach:** Boats are outside effective gun range. Players close distance and judge the other player's intent.
2. **Long-range fire:** Guns can reach, but hits are unlikely and may cause little damage.
3. **Effective open-water range:** Around 1,800 meters is the current estimate for likely hits. Maintaining or changing this range is a central skill.

The estimated maximum gun range is about 7.4 km. These weapon values are reference inputs for choosing game scale. They are not yet validated requirements for simulation accuracy.

The reference boat is a Marine Protector-class vessel, about 27 meters long and 6 meters wide, with a stated top speed near 46 km/h. At top speed it travels its hull length in about 2.1 seconds. These dimensions and speed inform scale, but the play area and camera do not need to use literal world scale.

The display target may be 1080p at 60 Hz. The boats must remain easy to see. A full 16 km by 9 km board shown at literal scale would make them too small, so camera and game scale must be selected for playability.

## Controls

Each boat has four analog controls and one momentary fire control:

- Hull yaw
- Thrust
- Gun yaw, within limits
- Gun pitch, within limits
- Fire

Analog input expresses normalized intent. For example, more stick deflection requests a larger share of the maximum available actuator force. The maximum can depend on game state. The ROM defines how its controls map to simulation actions.

The player steers the boat and controls the guns. Gun aiming is manual through gun yaw and pitch. Dual-stick gamepad support is a later input goal, not a prerequisite for the initial data model.

## Game Data

Separate reusable asset definitions from changing runtime instances.

- Both boats use the same untextured isosceles-triangle mesh. The triangle apex points along the boat's forward direction. Each boat instance supplies its player's color.
- The web and local renderers use the triangle mesh. The 2D terminal renderer uses a small Unicode triangle glyph, oriented to match the boat heading.
- A boat instance references a player and a boat asset. Its changing data includes position, velocity, integrity, ammunition, hull heading, and gun angles.
- Integrity and bullet damage use the same `DamageUnit` unit.
- A bullet instance references its asset and owner. Its changing data includes position and velocity. It carries damage data and has a path over each simulation update for collision testing.
- Land is excluded from the first scene. Later obstacle instances need placement and collision data as well as an asset reference. The same land geometry can provide obstacles and cover from fire.

## Data Organization

The ROM is exploring separate data tables for different fields and entity kinds, with typed IDs used to connect records. For example, boat identity and integrity can be stored in separate columns, and a `BoatId` lookup can find the matching row.

Each store owns its columns and any derived lookup indexes. Store methods perform insert, update, and removal so columns remain aligned and indexes remain correct. The current boat table uses a `HashMap<BoatId, usize>` index with vectors for the data columns. Hash-map lookup is average-case constant time. Full simulation passes should iterate the relevant vectors directly rather than scan the index.

This is an evolving design. Choose each layout for its consumers and measured workload. Do not treat a general-purpose ECS or a specific array layout as a fixed requirement for the ROM.

## Proposed Simulation Update Flow

The steps below describe target behavior. The current simulation only maps actions to log output and increments its tick counter.

The authority update clock is separate from the session's redraw clock. A redraw displays current session state. It does not define an authority step.

For one authority update:

1. Queue input signals from both players.
2. The ROM reduces each player's signals into its own tick intent. The runtime does not impose a shared action-reduction policy on other simulations.
3. Apply movement intent as force or torque. Larger analog input requests a larger portion of the current maximum. Mechanics updates velocity and caps its magnitude.
4. Apply gun control and fire intent. Firing creates a bullet near the boat's bow, with initial velocity based on its forward direction and the selected game rule.
5. Advance boats and bullets for the simulation time interval.
6. Resolve collisions and other interactions, then produce a batch of updates.
7. The server can queue published batches, but `SessionData` does not dequeue or apply them yet.

Batch application, validation, rollback, and recovery are not implemented.

## Proposed Collision Direction

The current ROM has no collision implementation. This section records design intent only.

Collision work has a broadphase and a narrowphase:

- The broadphase selects nearby candidate pairs.
- The narrowphase checks whether each candidate pair actually collides and determines the interaction.

A uniform grid is the current first candidate for the 2D world. It divides space into equal-sized cells and indexes entities in every cell overlapped by their collision bounds. It must handle negative coordinates with floor-based cell mapping, multi-cell objects, duplicate candidate pairs, and exact cell boundaries without missing possible contacts.

Fast bullets require swept collision checks over the path between their old and new positions. Their broadphase query must include cells touched by that path, expanded by the bullet's collision radius. Checking only the final position can miss an impact between updates.

Bullet-versus-bullet interception is a desired possibility for play. Streams can create dense local regions near boats. If active bullet counts grow substantially, measure candidate-pair counts, busiest-cell occupancy, and broadphase time. A finer or hierarchical partition is a later option if dense cells dominate the cost. Do not perform all-pairs bullet checks at large counts.

## Open Decisions

- Exact play-area dimensions, camera behavior, and whether both boats remain visible throughout the approach.
- Starting distance and time spent in approach, long-range, and effective-range phases.
- Whether bullet velocity includes the firing boat's velocity, and how projectile lifetime and speed are set.
- Whether bullet interceptions remove both bullets, only the intercepted bullet, or have another outcome.
- Exact collision shapes and swept-path narrowphase method for boats, bullets, and future land.
- Grid cell sizing and any partition refinement, based on representative measurements.
- Exact force, torque, speed, gun-traverse, gun-elevation, reload, integrity, ammunition, and damage values.