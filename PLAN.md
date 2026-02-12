# Game of Life Platformer — Implementation Plan

## Dependencies (all confirmed Bevy 0.18 compatible)

- `bevy = "0.18"` (default features — includes 2D, windowing, WASM support)
- `avian2d = "0.5"` (ECS-native 2D physics)
- `bevy-tnua = "0.30"` (floating character controller)
- `bevy-tnua-avian2d = "0.10"` (Tnua/Avian glue)
- `bevy_ecs_ldtk = "0.14"` (LDtk level loading)

## Project Structure

```
meaningless/
  src/
    main.rs        — App setup, DefaultPlugins, window, top-level plugin registration
    player.rs      — TnuaScheme, player spawn, input system
    camera.rs      — Smooth-follow camera
    level.rs       — LDtk loading, IntGrid -> grid state, platform entity spawning
    gol.rs         — Game of Life tick timer, simulation, spawn/despawn/death logic
  assets/
    levels/
      world.ldtk   — LDtk project (user creates this in the LDtk app)
  index.html       — Trunk WASM entry point (640x640 canvas, pixelated rendering)
  Trunk.toml       — Trunk build config
  Cargo.toml
```

No `lib.rs`. Just `main.rs` with `mod` declarations. Each module exposes a `pub fn *_plugin_fn(app: &mut App)`.

## Status

- Milestone 1 — DONE (with known issue: Tnua float spring still feels off, needs tuning)
- Milestone 2 — next
- Milestone 3 — pending
- Milestone 4 — pending

## Milestone 1 — Skeleton: Window, WASM, Camera, Player Rectangle (DONE)

**Goal:** Green rectangle on screen, compiles to WASM, camera follows it.

**What was built:**

- `Cargo.toml` with all deps + dev profile optimizations
- `index.html` — 640x640 canvas, centered, pixelated rendering
- `Trunk.toml` — basic build config
- `main.rs` — App with DefaultPlugins (640x640 window, nearest-neighbor image filtering), registers all plugin functions
- `player.rs` — `Player` marker, green `Sprite` as child entity offset down by float gap, `RigidBody::Dynamic` + `Collider`, `TnuaController` with `PlayerScheme`, WASD movement + J to jump, `LockedAxes`, `TransformInterpolation`, `SpeculativeMargin`
- `camera.rs` — `GameCamera` marker, snaps to player position in `PostUpdate` (no lerp to avoid flicker with physics interpolation)
- `level.rs` — Hardcoded ground + 5 platforms with `RigidBody::Static` + `Collider::rectangle`
- `gol.rs` — Empty plugin stub

**Controls:** A/D to move, J to jump

**Known issue:** Tnua floating character controller spring tuning still not perfect — character can clip slightly or feel floaty. May need to revisit approach (consider simpler kinematic character controller if Tnua keeps fighting us).

## Milestone 2 — LDtk Integration

**Goal:** Design levels in LDtk, load them in game, walk on platforms.

**LDtk project setup** (user creates in the LDtk app):

- 32px grid size
- IntGrid layer called "Cells" with values:
  - `1` = permanent cell (always alive, drawn black)
  - `2` = GoL cell (subject to simulation, drawn dark grey)
- Entity layer called "Entities" with a `PlayerSpawn` entity

**What to build:**

- `level.rs` — Load `assets/levels/world.ldtk` via `LdtkPlugin`. Register `LdtkIntCell` bundles for IntGrid values 1 and 2 — each spawns a `Sprite` + `RigidBody::Static` + `Collider::rectangle(32, 32)` with appropriate color. Register `LdtkEntity` for `PlayerSpawn`. System that reads `PlayerSpawn` position and spawns the player there. Remove the hardcoded ground
- `gol.rs` — `GridState` resource built from querying all `IntGridCell` + `GridCoords` entities after level load. Maps each cell to `CellKind::Permanent` or `CellKind::Dynamic(alive: bool)`. No simulation yet

**Test:** Design a level in LDtk with permanent and dynamic platforms, save, `cargo run`. Player spawns at PlayerSpawn, can walk/jump on all platforms. Permanent = black, dynamic = dark grey. Logs show grid state dimensions and cell counts.

## Milestone 3 — Game of Life Simulation

**Goal:** Platforms evolve according to GoL rules. Death on crush.

**What to build:**

- `gol.rs` — `GolTimer` resource (configurable, default 0.75s). Each tick:
  1. Snapshot current grid state
  2. Apply standard GoL rules (B3/S23) — permanent cells count as alive neighbors but never change
  3. Diff old vs new state
  4. For newly dead cells: hide platform entity (remove `Collider`, set `Visibility::Hidden`)
  5. For newly born cells: show platform entity (add `Collider`, set `Visibility::Inherited`)
  6. **Crush detection:** after spawning new platforms, check overlap with player collider. If overlap, trigger death
- Death state: log "PLAYER DIED" + respawn at `PlayerSpawn` after short delay
- `info!` logs for each tick: cells born, cells died, total alive

**Test:** Load a level with GoL patterns (glider, blinker). Watch platforms appear/disappear every 0.75s. Try to survive. Try getting crushed.

## Milestone 4 — Double Jump + Coyote Time + Polish

**Goal:** Game feels good to play.

**What to build:**

- `player.rs` — Double jump (track air jump count, allow one additional jump while airborne). Tune walk/jump configs for:
  - Generous coyote time
  - Variable-height jump (hold = higher, tap = lower — Tnua does this by default)
- `camera.rs` — Tune follow, maybe add slight lookahead in movement direction
- `gol.rs` — Tune tick rate, add visual feedback before tick (stretch goal)
- General: tune gravity, movement speed, jump height until it feels right

**Test:** Full gameplay loop. Run, jump, double jump across evolving GoL platforms.

## Key Architecture Decisions

- **Platform entities are ECS entities with Collider** — toggled via `Visibility` + adding/removing `Collider` rather than spawn/despawn, to avoid entity churn
- **GoL state is a `Resource` grid** — canonical source of truth, platform entities sync to it each tick
- **Player death is an event** — `PlayerDeathEvent` so we can later add animations, screen shake, etc.
- **Gravity:** `Gravity(Vec2::new(0.0, -800.0))` — Avian default (9.81) is too weak for pixel units
- **Tnua float height** must be > half the player's collider height

## Logging Strategy

- `info!` on app startup: window size, tile size, gravity
- `info!` on level load: grid dimensions, permanent cell count, dynamic cell count, player spawn position
- `info!` on each GoL tick: tick number, cells born, cells died, total alive
- `warn!` on player death with cause (crush / fell off screen)
- `debug!` on player state changes (grounded, airborne, double-jumped)
