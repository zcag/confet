# confet

GPU-rendered confetti overlay for Linux (Wayland) and macOS.

## Demo

<!-- record: confetti name=hero scale=1080 crf=18 fps=20 -->

<p align="center">
  <img src="https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/hero.gif" width="600">
</p>

<!-- record: confetti -->
<!-- record: pop -->
<!-- record: snow -->
<!-- record: rain -->
<!-- record: sparkle -->

| confetti | pop | snow | rain | sparkle |
|----------|-----|------|------|---------|
| ![confetti](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/confetti.gif) | ![pop](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/pop.gif) | ![snow](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/snow.gif) | ![rain](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/rain.gif) | ![sparkle](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/sparkle.gif) |

<!-- record: lava -->
<!-- record: sakura -->
<!-- record: matrix -->

| lava | sakura | matrix |
|------|--------|--------|
| ![lava](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/lava.gif) | ![sakura](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/sakura.gif) | ![matrix](https://github.com/zcag/confet/raw/refs/heads/master/.github/assets/matrix.gif) |

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

confet returns immediately and animates in the background. Pass `--wait` to
block until it finishes.

```sh
confet                    # default confetti
confet snow               # built-in type
confet lava               # built-in profile (no config needed)
confet -t pop -n 500      # type with overrides
confet --size 2           # chunkier particles
confet --info gold        # what would 'gold' actually do?
confet --init             # create config file
```

### Examples

Celebrate after a build:
```sh
cargo build --release && confet
make && confet gold
```

confet returns as soon as the animation starts, so it never holds up a chain:
```sh
make && confet gold && ./run-tests.sh
```

Use `--wait` if you need the animation to finish first — recording it, say:
```sh
confet --wait fireworks
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

## Sound

Off by default. `--sound` on its own plays the sound that fits the animation
type; a name or a path to a `.wav` overrides it.

```sh
confet --sound              # the type's own sound
confet --sound cork         # a built-in by name
confet --sound ~/pop.wav    # your own file
confet gold --mute          # force silence over a config that sets one
```

Built-in: `cork`. Bundled sounds are CC0 — see
[assets/sounds/CREDITS.md](assets/sounds/CREDITS.md).

Playback shells out to whatever the platform already has (`afplay` on macOS;
`pw-play`, `paplay`, `aplay` or `ffplay` on Linux), so confet needs no audio
library and no extra build dependencies. If none of them are installed, the
animation runs silently.

To make it always-on, set `sound` in the config — top level or per profile:

```toml
sound = "auto"          # every run plays its type's sound

[profiles.gold]
sound = "cork"
```

## Config

Generate a default config with `confet --init`. It is written to
`$XDG_CONFIG_HOME/confet/config.toml`, defaulting to `~/.config/confet/config.toml`
on Linux **and** macOS; `confet --info` prints the path actually in use.

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

Settings for a single animation type go in a `[types.<name>]` section — unlike
top-level values, they leave every other type's defaults alone:

```toml
[types.confetti]
particles = 3000        # a bigger default burst, without touching snow
```

**Priority:** CLI flags > profile > `[types.<name>]` > top-level config > type defaults.

Config profiles override built-in profiles with the same name.

## CLI reference

| Flag | Description | Default |
|------|-------------|---------|
| `[PROFILE]` | Profile name or animation type | confetti |
| `-t, --type` | Animation type | confetti |
| `-s, --shape` | Particle shape (rect, circle, triangle, mixed) | varies by type |
| `-n, --particles` | Number of particles | varies by type |
| `-d, --duration` | Animation length (secs); `0` = until every particle is off screen (60s ceiling) | varies by type |
| `-g, --gravity` | Gravity strength | varies by type |
| `--drag` | Air resistance (0-1) | varies by type |
| `--speed-min` | Min launch speed | varies by type |
| `--speed-max` | Max launch speed | varies by type |
| `--spread` | Horizontal spread | varies by type |
| `--fade` | Fade-out duration (secs); `0` = no fade | varies by type |
| `--size` | Particle size multiplier | 1 |
| `-c, --colors` | Hex colors, comma-separated | varies by type |
| `-w, --wait` | Block until the animation finishes | off |
| `--sound [NAME\|PATH]` | Play a sound; bare = the type's own | off |
| `--mute` | Force silence, overriding the config | — |
| `--info` | Print the resolved settings and exit | — |
| `--init` | Create default config file | — |

## License

MIT
