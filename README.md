# confet

GPU-rendered confetti overlay for Linux (Wayland) and macOS.

## Demo

<!-- record: confetti name=hero scale=1080 crf=18 -->

<p align="center">
  <video src="https://github.com/zcag/confet/raw/main/.github/assets/hero.mp4" autoplay muted loop playsinline width="600"></video>
</p>

<!-- record: confetti -->
<!-- record: pop -->
<!-- record: snow -->
<!-- record: rain -->
<!-- record: sparkle -->

| confetti | pop | snow | rain | sparkle |
|----------|-----|------|------|---------|
| <video src="https://github.com/zcag/confet/raw/main/.github/assets/confetti.mp4" autoplay muted loop playsinline width="180"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/pop.mp4" autoplay muted loop playsinline width="180"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/snow.mp4" autoplay muted loop playsinline width="180"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/rain.mp4" autoplay muted loop playsinline width="180"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/sparkle.mp4" autoplay muted loop playsinline width="180"></video> |

<!-- record: lava -->
<!-- record: sakura -->
<!-- record: matrix -->

| lava | sakura | matrix |
|------|--------|--------|
| <video src="https://github.com/zcag/confet/raw/main/.github/assets/lava.mp4" autoplay muted loop playsinline width="240"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/sakura.mp4" autoplay muted loop playsinline width="240"></video> | <video src="https://github.com/zcag/confet/raw/main/.github/assets/matrix.mp4" autoplay muted loop playsinline width="240"></video> |

## Install

### Linux (Wayland)

Requires GTK4 and gtk4-layer-shell.

```sh
# Arch
sudo pacman -S gtk4 gtk4-layer-shell

# Ubuntu/Debian
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
```

### macOS

Requires GTK4 via Homebrew.

```sh
brew install gtk4
```

### Binary

```sh
cargo binstall confet
# or
cargo install confet
```

### From source

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

### Examples

Celebrate after a build:
```sh
cargo build --release && confet
make && confet gold
```

Xcode build hook:
```sh
xcodebuild -project MyApp.xcodeproj && confet lava
```

After a long-running command:
```sh
./train-model.sh; confet fireworks
sleep 3600 && confet sakura
```

CI/deploy success notification:
```sh
ssh prod "deploy.sh" && confet gold
```

Git hook (`.git/hooks/post-commit`):
```sh
#!/bin/sh
confet -n 300 -d 1.5
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
