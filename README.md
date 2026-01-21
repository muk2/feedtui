# feedtui

A configurable terminal dashboard for browsing news, stocks, sports, and more - with a virtual pet companion!

## Features

- **Hacker News** - Browse top, new, and best stories
- **Stock Ticker** - Track your portfolio in real-time
- **RSS Feeds** - Subscribe to your favorite news sources
- **Sports Scores** - Follow NBA, NFL, EPL, and more
- **Tui** - Your virtual companion creature that levels up as you use the terminal!

## Installation

```bash
cargo install feedtui
```

Or build from source:

```bash
git clone https://github.com/yourusername/feedtui
cd feedtui
cargo build --release
```

## Configuration

Create a `.feedtui` folder in your home directory and add a `config.toml` file:

```bash
mkdir -p ~/.feedtui
cp config.example.toml ~/.feedtui/config.toml
```

Edit the config to customize your dashboard layout and feeds.

## Meet Tui!

Tui (pronounced "chew-ee") is your virtual companion creature that lives in your terminal. The more you use feedtui, the more Tui grows!

### Features

- **10 Different Species** - Choose from Blob, Bird, Cat, Dragon, Fox, Owl, Penguin, Robot, Spirit, or Octopus
- **Leveling System** - Earn XP just by using the terminal
- **Skill Tree** - Unlock skills with points earned from leveling up
- **Outfits** - Customize Tui with unlockable outfits like Hacker, Wizard, Ninja, Astronaut, and more
- **Moods** - Tui reacts to how often you visit
- **Persistent Progress** - Your creature's progress is saved automatically

### Keybindings

| Key | Action |
|-----|--------|
| `t` | Toggle Tui menu |
| `Tab` / `Shift+Tab` | Switch between menu tabs / widgets |
| `j` / `k` or arrows | Navigate lists |
| `Enter` | Select/purchase items in menu |
| `r` | Refresh feeds |
| `q` | Quit |

### Skill Tree

Unlock skills by spending points:

- **Greeting** (Free) - Tui greets you on startup
- **News Digest** (10 pts) - Highlights important news
- **Stock Alert** (15 pts) - Alerts on significant movements
- **Quick Learner** (15 pts) - +10% XP gain
- **Speed Read** (20 pts) - Faster feed refresh
- **Fast Learner** (30 pts) - +25% XP gain
- **Cosmic Insight** (50 pts) - Trending topic insights
- **Fire Breath** (40 pts) - Cosmetic fire animation
- **Omniscience** (100 pts) - Maximum XP boost

### Outfit Unlocks

Outfits unlock as you level up:

| Level | Outfit |
|-------|--------|
| 1 | Default |
| 5 | Hacker |
| 10 | Wizard |
| 15 | Ninja |
| 20 | Astronaut |
| 25 | Robot |
| 30 | Dragon |
| 50 | Legendary |

## Example Config

```toml
[general]
refresh_interval_secs = 60
theme = "dark"

# Tui - Your companion creature!
[[widgets]]
type = "creature"
title = "Tui"
show_on_startup = true
position = { row = 0, col = 0 }

# Hacker News
[[widgets]]
type = "hackernews"
title = "Hacker News"
story_count = 10
story_type = "top"
position = { row = 0, col = 1 }

# Stocks
[[widgets]]
type = "stocks"
title = "Portfolio"
symbols = ["AAPL", "GOOGL", "MSFT"]
position = { row = 1, col = 0 }
```

## License

MIT
