# gitwatch-tui

A terminal dashboard that shows every open pull request you are involved in across all your repositories, and tells you which ones need your attention right now.

It lives in your terminal, refreshes itself, and sorts your PRs by how urgent they are - so you stop refreshing GitHub in ten browser tabs.

<!-- Replace this line with a GIF of the live dashboard in action -->
![gitwatch demo](docs/demo.gif)

## What it does

- Collects all your open PRs (where you are the author) from every repository, in a single request.
- Sorts them by urgency: NOW, SOON, LOW.
- Refreshes automatically every 30 seconds and highlights anything that changed since the last check.
- Lets you group repositories (for example all services of one project) and filter by group or by repository.
- Read only. Your token never leaves your machine and nothing is changed on GitHub.

## Requirements

You need two things installed:

1. Rust (the `cargo` command). Install it from https://rustup.rs
2. GitHub CLI (the `gh` command). gitwatch reuses the login you already have in `gh`, so you do not need to create any token by hand.

If `gh` is missing when you start gitwatch, it prints the exact install command for your operating system.

## Step 1: log in to GitHub CLI

This is needed for both install options below, one time only:

```
gh auth login
```

To install GitHub CLI itself: on Arch Linux `sudo pacman -S github-cli`, on macOS `brew install gh`, on Windows `winget install --id GitHub.cli`.

## Step 2: install gitwatch

Pick one of the two options.

### Option A: download a prebuilt binary (no Rust needed)

This is the easiest way and does not require Rust.

1. Open the releases page: https://github.com/AbuZhuma/gitwatch-tui/releases
2. Download the file for your system:
   - Linux: `gitwatch-x86_64-unknown-linux-musl.tar.gz`
   - macOS (Apple Silicon): `gitwatch-aarch64-apple-darwin.tar.gz`
   - macOS (Intel): `gitwatch-x86_64-apple-darwin.tar.gz`
   - Windows: `gitwatch-x86_64-pc-windows-msvc.zip`
3. Extract it and move the `gitwatch` program to a folder on your PATH. On Linux or macOS:

   ```
   tar -xzf gitwatch-x86_64-unknown-linux-musl.tar.gz
   sudo mv gitwatch /usr/local/bin/
   ```

   On Windows, unzip the file and put `gitwatch.exe` in a folder that is on your PATH.

Now run it from anywhere:

```
gitwatch
```

### Option B: build from source (needs Rust)

Install Rust from https://rustup.rs first, then:

```
git clone https://github.com/AbuZhuma/gitwatch-tui
cd gitwatch-tui
cargo install --path .
```

This puts a `gitwatch` command in `~/.cargo/bin`, which is already on your PATH. Run it from anywhere:

```
gitwatch
```

To just try it without installing, run `cargo run --release` inside the project folder.

## Keys

The whole app is controlled with the arrow keys.

| Key | Action |
|-----|--------|
| Up / Down | Move the cursor. At the top of the PR list, Up jumps into Groups and Repos; at the bottom of Groups, Down jumps into the PR list |
| Enter | In Groups and Repos: filter the PR list by the selected group or repository |
| Right | In Groups and Repos: open the details of a group. In the PR list: open the details of a PR |
| Left | In the PR list: close the details, or go back to Groups and Repos |
| n | Create a new group |
| Backspace | Delete the selected group |
| r | Refresh now |
| q | Quit |

## Groups

A group is a named set of repositories, for example all the repositories of one project.

- Press `n`, type a name, press Enter, then pick repositories with Space and press Enter to save.
- Select a group and press Enter to show only its pull requests.
- Select a group and press Right to see all of its repositories and how many PRs each one has, even the ones without any open PR.
- Press Backspace on a group to delete it.

Groups are saved on your machine in the gitwatch config file and are loaded again next time you start it. On Linux this file is `~/.config/gitwatch/config.toml`.

## How urgency is decided

There is no magic here. gitwatch uses a small set of fixed rules that are easy to explain:

NOW (needs action):
- CI failed on your PR.
- Your PR is approved, has no conflicts, and is not a draft, so it is ready to merge.
- Someone mentioned you in a new comment since the last refresh.

SOON (will need action):
- Your PR has a conflict with its base branch.
- A reviewer requested changes.
- Your PR has had no review and has been open for 2 days or more.

LOW (background):
- Everything else.

The list is ordered from NOW to LOW, and within each level the most recently updated PRs come first.

## Roadmap

Planned for later versions:

- Configurable prioritization rules through `gitwatch.toml`, with arbitrary conditions and not only thresholds.
- Support for other platforms such as GitLab and Bitbucket.
- Shareable rule presets that people can publish and reuse.

## License

MIT. See [LICENSE](LICENSE).
