# Game of Life Platformer — Implementation Plan

## Dependencies (Bevy 0.18)

- `bevy = "0.18"` (default features — includes 2D, windowing, WASM support)
- `avian2d = "0.5"` (ECS-native 2D physics — used for collision detection via `SpatialQuery`)
- `bevy_ecs_ldtk = "0.14"` (LDtk level loading — not yet used)

## Project Structure

```
meaningless/
  src/
    main.rs        — App setup, DefaultPlugins, window, top-level plugin registration
    player.rs      — Custom kinematic player controller with shape-cast collision
    camera.rs      — Pixel-perfect camera with deadzone follow
    level.rs       — LDtk level loading, cell bundles, player spawn
    gol.rs         — Game of Life simulation, GridState resource
    game_state.rs  — GameState enum, LevelGrid, menus (Milestone 5)
  assets/
    levels/
      play.ldtk    — LDtk project with multiple levels
  index.html       — Trunk WASM entry point (640x640 canvas, pixelated rendering)
  Trunk.toml       — Trunk build config
  Cargo.toml
```

No `lib.rs`. Just `main.rs` with `mod` declarations. Each module exposes a `pub fn *_plugin_fn(app: &mut App)`.

## Status

- Milestone 1 — DONE
- Milestone 2 — DONE
- Milestone 3 — next
- Milestone 4 — pending
- Milestone 5 — pending

## Milestone 1 — Skeleton: Window, Camera, Player Rectangle (DONE)

**Goal:** Green rectangle on screen, pixel-perfect rendering, camera follows player.

**What was built:**

- `Cargo.toml` with deps + dev profile optimizations (opt-level 0 for local, 3 for dependencies)
- `main.rs` — App with DefaultPlugins (640x640 window, nearest-neighbor image filtering via `ImagePlugin::default_nearest()`), Avian `PhysicsPlugins`, registers all plugin functions. Constants: `INTERNAL_SIZE = 640`, `TILE_SIZE = 32`
- `player.rs` — Custom kinematic character controller:
  - `Player` marker + `PlayerState` component tracking velocity, grounded state, timers
  - Shape-cast collision using Avian's `SpatialQuery` (no rigidbody on player — fully kinematic)
  - Acceleration-based horizontal movement (different accel/decel for ground vs air)
  - Variable-height jump (release early = lower jump via gravity multiplier)
  - Coyote time (0.16s grace period after leaving ground)
  - Jump buffering (0.16s input buffer before landing)
  - Collision skin to prevent tunneling
- `camera.rs` — Pixel-perfect rendering setup:
  - Two-camera system: `InGameCamera` renders to an offscreen texture at internal resolution, `OuterCamera` displays that texture scaled up
  - `RenderLayers` to separate pixel-perfect content from the upscaled canvas
  - Deadzone-based follow: camera stays still while player is within a box (16px horizontal, 24px vertical), only moves when player pushes past edges
  - Smooth catch-up when following (lerp with `CATCH_UP_SPEED = 8.0`) to avoid jarring snaps
- `level.rs` — Hardcoded ground + 5 platforms with `RigidBody::Static` + `Collider::rectangle`
- `gol.rs` — Empty plugin stub

**Controls:** A/D to move, J to jump (hold for higher jump)

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

## Milestone 4 — Double Jump + Polish

**Goal:** Game feels good to play.

**What to build:**

- `player.rs` — Double jump (track air jump count, allow one additional jump while airborne)
- `gol.rs` — Tune tick rate, add visual feedback before tick (stretch goal)
- General: tune movement constants until it feels right

**Test:** Full gameplay loop. Run, jump, double jump across evolving GoL platforms.

## Milestone 5 — Menus & Grid Progression

**Goal:** Main menu, grid-based level select, progress persistence.

**Game States:**

```rust
#[derive(States, Default, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    LevelSelect,
    Playing,
}
```

All gameplay systems get `.run_if(in_state(GameState::Playing))`. Menu/UI systems run in their respective states. `OnEnter`/`OnExit` schedules handle setup and teardown.

**Level Grid Resource:**

```rust
#[derive(Resource)]
struct LevelGrid {
    width: usize,
    height: usize,
    unlocked: HashSet<IVec2>,
    completed: HashSet<IVec2>,
    current: Option<IVec2>,
}
```

- Grid position `(x, y)` maps to LDtk level index `y * width + x`
- Starting state: only `(0, 0)` unlocked
- Completing a level unlocks 4-adjacent neighbors (up/down/left/right)

**What to build:**

- `game_state.rs` — `GameState` enum, `LevelGrid` resource, state transition systems
- Menu screen: simple "Play" button → `LevelSelect`
- Level select screen: render grid, locked levels greyed out, completed levels show checkmark, arrow key navigation, Enter to select
- Win condition: add `Goal` entity in LDtk, trigger `LevelCompleteEvent` when player touches it
- On level complete: call `grid.complete_level(pos)`, transition to `LevelSelect`
- Persistence: serialize `unlocked` + `completed` to JSON. Native: write to config dir. WASM: browser localStorage via `bevy_pkv` or `web-sys`
- Backspace to restart current level (despawn + respawn level entities)

**Test:** Start game, see menu. Enter level select, only (0,0) available. Beat level, adjacent levels unlock. Quit and relaunch, progress persists.

## Key Architecture Decisions

- **Custom kinematic controller** — Player uses shape-cast collision via `SpatialQuery` rather than a physics rigidbody. This gives precise control over movement feel without fighting the physics engine
- **Pixel-perfect rendering** — Two-camera setup renders game at internal resolution then scales up, ensuring crisp pixels at any window size
- **Deadzone camera** — Camera only moves when player pushes past edges of an invisible box, with smooth lerp catch-up. Feels more grounded than pure lerp follow
- **Platform entities are ECS entities with Collider** — will be toggled via `Visibility` + adding/removing `Collider` rather than spawn/despawn, to avoid entity churn
- **GoL state will be a `Resource` grid** — canonical source of truth, platform entities sync to it each tick
- **Player death will be an event** — `PlayerDeathEvent` so we can later add animations, screen shake, etc.

## Player Movement Constants

```rust
MOVE_SPEED: 200.0      // Target horizontal velocity
GROUND_ACCEL: 8000.0   // Acceleration when grounded
GROUND_DECEL: 8000.0   // Deceleration when grounded (no input)
AIR_ACCEL: 5000.0      // Acceleration in air
AIR_DECEL: 2400.0      // Deceleration in air (no input)
JUMP_VELOCITY: 365.0   // Initial upward velocity on jump
GRAVITY: -900.0        // Downward acceleration
JUMP_CUT_MULT: 2.2     // Gravity multiplier when jump released early
MAX_FALL_SPEED: -400.0 // Terminal velocity
COYOTE_TIME: 0.16      // Grace period after leaving ground
JUMP_BUFFER: 0.16      // Input buffer before landing
```

## Logging Strategy

- `info!` on level load: platform count, player spawn position
- `info!` on each GoL tick: tick number, cells born, cells died, total alive
- `warn!` on player death with cause (crush / fell off screen)
