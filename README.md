# confet

GPU-rendered confetti overlay for Wayland.

## Demo

<!-- record: confetti -->
<!-- record: cannon -->
<!-- record: pop -->
<!-- record: fireworks -->

| confetti | cannon | pop | fireworks |
|----------|--------|-----|-----------|
| <video src=".github/assets/confetti.mp4" width="200"> | <video src=".github/assets/cannon.mp4" width="200"> | <video src=".github/assets/pop.mp4" width="200"> | <video src=".github/assets/fireworks.mp4" width="200"> |

<!-- record: snow -->
<!-- record: rain -->
<!-- record: sparkle -->
<!-- record: drop -->

| snow | rain | sparkle | drop |
|------|------|---------|------|
| <video src=".github/assets/snow.mp4" width="200"> | <video src=".github/assets/rain.mp4" width="200"> | <video src=".github/assets/sparkle.mp4" width="200"> | <video src=".github/assets/drop.mp4" width="200"> |

<!-- record: lava -->
<!-- record: matrix -->
<!-- record: sakura -->
<!-- record: aurora -->
<!-- record: gold -->
<!-- record: balloon -->

| lava | matrix | sakura | aurora | gold | balloon |
|------|--------|--------|--------|------|---------|
| <video src=".github/assets/lava.mp4" width="150"> | <video src=".github/assets/matrix.mp4" width="150"> | <video src=".github/assets/sakura.mp4" width="150"> | <video src=".github/assets/aurora.mp4" width="150"> | <video src=".github/assets/gold.mp4" width="150"> | <video src=".github/assets/balloon.mp4" width="150"> |

## Install

### cargo-binstall (prebuilt binary)

```sh
cargo binstall confet
```

### cargo install

```sh
cargo install confet
```

### From source

Requires GTK4 and gtk4-layer-shell development libraries.

```sh
git clone https://github.com/zcag/confet
cd confet
cargo build --release
```

## Usage

```sh
confet                    # default confetti
confet snow               # built-in type
confet lava               # built-in profile (no config needed)
confet -t pop -n 500      # type with overrides
confet --init             # create config file
```

## Types

| Type | Shape | Description |
|------|-------|-------------|
| `confetti` | rect | Burst from bottom corners (default) |
| `cannon` | rect | Single burst from center bottom |
| `pop` | mixed | Radial burst from screen center |
| `fireworks` | circle | Explosion in the upper sky |
| `snow` | circle | Gentle drift from the top |
| `rain` | rect | Fast vertical streaks |
| `sparkle` | circle | Twinkling particles at random positions |
| `drop` | mixed | Particles dropping from above |

Each type has its own default physics, shape, and colors.

## Built-in profiles

These work out of the box — no config file needed.

| Profile | Type | Description |
|---------|------|-------------|
| `lava` | pop | Red/orange/yellow explosion |
| `matrix` | rain | Green digital rain |
| `sakura` | snow | Pink cherry blossom petals |
| `aurora` | sparkle | Northern lights shimmer |
| `gold` | cannon | Golden circles from center |
| `balloon` | drop | Rainbow drops from above |

## Config

Generate a default config with `confet --init` (creates `~/.config/confet/config.toml`):

```toml
# Top-level settings override type defaults
particles = 1500
duration = 2.5
colors = ["#ff2d87", "#2d8cff", "#2dff6d", "#ffd02d"]

# Named profiles: confet <name>
[profiles.lava]
type = "pop"
colors = ["#ff2200", "#ff6600", "#ffaa00", "#ffdd00"]

[profiles.sakura]
type = "snow"
particles = 250
duration = 8.0
colors = ["#ffb7c5", "#ff69b4", "#ffc0cb", "#ffffff"]
```

**Priority:** CLI flags > profile settings > top-level config > type defaults.

Config profiles override built-in profiles with the same name.

## CLI reference

| Flag | Description | Default |
|------|-------------|---------|
| `[PROFILE]` | Profile name or animation type | confetti |
| `-t, --type` | Animation type | confetti |
| `-s, --shape` | Particle shape (rect, circle, mixed) | varies by type |
| `-n, --particles` | Number of particles | varies by type |
| `-d, --duration` | Animation length (secs) | varies by type |
| `-g, --gravity` | Gravity strength | varies by type |
| `--drag` | Air resistance (0-1) | varies by type |
| `--speed-min` | Min launch speed | varies by type |
| `--speed-max` | Max launch speed | varies by type |
| `--spread` | Horizontal spread | varies by type |
| `--fade` | Fade-out duration (secs) | varies by type |
| `-c, --colors` | Hex colors, comma-separated | varies by type |
| `--init` | Create default config file | — |

## License

MIT
