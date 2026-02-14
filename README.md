# confet

GPU-rendered confetti overlay for Wayland.

## Installation

### cargo-binstall (prebuilt binary)

```sh
cargo binstall confet
```

### From source

Requires GTK4 and gtk4-layer-shell development libraries.

```sh
cargo install confet
```

## Usage

```sh
confet                    # launch with defaults
confet -n 500 -d 2        # 500 particles, 2 second duration
confet --colors '#ff0000,#00ff00,#0000ff'   # custom colors
```

## CLI options

| Flag | Description | Default |
|------|-------------|---------|
| `-n, --particles` | Number of particles | 1200 |
| `-d, --duration` | Animation length (secs) | 3.5 |
| `-g, --gravity` | Gravity strength | 1000 |
| `--drag` | Air resistance (0-1) | 0.985 |
| `--speed-min` | Min launch speed | 1200 |
| `--speed-max` | Max launch speed | 3000 |
| `--spread` | Horizontal spread | 200 |
| `--fade` | Fade-out duration (secs) | 0.6 |
| `-c, --colors` | Hex colors, comma-separated | 9-color palette |
| `--init` | Create default config file | — |

## Config file

Generate a default config file with `confet --init`. This creates `~/.config/confet/config.toml` with all defaults:

```toml
particles = 800
duration = 5.0
gravity = 1200
drag = 0.98
speed_min = 1000
speed_max = 2500
spread = 150
fade = 0.8
colors = ["#ff0000", "#00ff00", "#0000ff"]
```

CLI flags override config file values, which override defaults.

## License

MIT
