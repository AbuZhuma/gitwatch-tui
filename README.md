# gitwatch-tui

> Lives in your terminal and taps you on the shoulder the moment something across your repos needs you — no more refreshing GitHub in 15 tabs.

A terminal live-dashboard that shows every open pull request you're involved in across all your repositories, and highlights which ones need action **right now** — failing CI, ready-to-merge, or a fresh @mention.

Read-only by design. Your token never leaves your machine.

## Status

🚧 Early development. Stage 0 (skeleton) is in place: the app boots into a TUI and quits cleanly. Auth, data and the live dashboard land in the following stages.

## Quick start

Requires a recent Rust toolchain.

```sh
git clone https://github.com/ok4ami/gitwatch-tui
cd gitwatch-tui
cargo run
```

Press `q` to quit.

> From stage 1 onward you'll also need the GitHub CLI logged in (`gh auth login`); the app reuses that token via `gh auth token`.

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |

More land as the dashboard grows.

## License

MIT — see [LICENSE](LICENSE).
