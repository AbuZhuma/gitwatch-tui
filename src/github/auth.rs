use std::io::ErrorKind;
use std::process::Command;

use crossterm::style::Stylize;

const INSTALL_URL: &str = "https://github.com/cli/cli#installation";

pub enum AuthError {
    NotInstalled,
    NotAuthenticated(String),
    Other(anyhow::Error),
}

enum Block {
    Text(String),
    Commands(Vec<String>),
}

pub struct Guidance {
    heading: String,
    blocks: Vec<Block>,
}

pub fn token() -> Result<String, AuthError> {
    let output = match Command::new("gh").args(["auth", "token"]).output() {
        Ok(output) => output,
        Err(e) if e.kind() == ErrorKind::NotFound => return Err(AuthError::NotInstalled),
        Err(e) => return Err(AuthError::Other(e.into())),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(AuthError::NotAuthenticated(stderr));
    }

    let token = String::from_utf8(output.stdout)
        .map_err(|e| AuthError::Other(e.into()))?
        .trim()
        .to_owned();

    if token.is_empty() {
        return Err(AuthError::NotAuthenticated(String::new()));
    }

    Ok(token)
}

impl AuthError {
    pub fn guidance(&self) -> Guidance {
        match self {
            AuthError::NotInstalled => Guidance {
                heading: "GitHub CLI (gh) is required, but it was not found.".to_owned(),
                blocks: install_blocks(),
            },
            AuthError::NotAuthenticated(detail) => Guidance {
                heading: "You are not signed in to the GitHub CLI.".to_owned(),
                blocks: sign_in_blocks(detail),
            },
            AuthError::Other(e) => Guidance {
                heading: format!("Authentication failed: {e:#}"),
                blocks: Vec::new(),
            },
        }
    }
}

impl Guidance {
    pub fn render(&self, color: bool) -> String {
        let marker = if color {
            "!".yellow().bold().to_string()
        } else {
            "!".to_owned()
        };
        let heading = if color {
            self.heading.clone().bold().to_string()
        } else {
            self.heading.clone()
        };
        let mut out = format!("\n  {marker}  {heading}\n");

        for block in &self.blocks {
            out.push('\n');
            match block {
                Block::Text(text) => {
                    out.push_str(&format!("  {text}\n"));
                }
                Block::Commands(commands) => {
                    let gutter = if color {
                        "│".dark_grey().to_string()
                    } else {
                        "│".to_owned()
                    };
                    for command in commands {
                        let command = if color {
                            command.clone().cyan().bold().to_string()
                        } else {
                            command.clone()
                        };
                        out.push_str(&format!("    {gutter} {command}\n"));
                    }
                }
            }
        }

        out.push('\n');
        out
    }
}

fn install_blocks() -> Vec<Block> {
    let mut blocks = vec![Block::Text("Install it:".to_owned())];

    if cfg!(target_os = "macos") {
        blocks.push(commands(&["brew install gh"]));
    } else if cfg!(target_os = "windows") {
        blocks.push(commands(&["winget install --id GitHub.cli"]));
        blocks.push(Block::Text(
            "Or with Scoop / Chocolatey: scoop install gh  •  choco install gh".to_owned(),
        ));
    } else if cfg!(target_os = "linux") {
        match linux_install_command() {
            Some(command) => blocks.push(commands(&command.lines().collect::<Vec<_>>())),
            None => blocks.push(Block::Text(
                "Install the \"gh\" (also packaged as \"github-cli\") package \
                 with your distribution's package manager."
                    .to_owned(),
            )),
        }
    } else {
        blocks.push(Block::Text(
            "Install the GitHub CLI for your operating system.".to_owned(),
        ));
    }

    blocks.push(Block::Text(format!("More options: {INSTALL_URL}")));
    blocks.push(Block::Text(
        "Then sign in and run gitwatch again:".to_owned(),
    ));
    blocks.push(commands(&["gh auth login"]));
    blocks
}

fn sign_in_blocks(detail: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    if !detail.is_empty() {
        blocks.push(Block::Text(detail.to_owned()));
    }
    blocks.push(Block::Text("Sign in, then run gitwatch again:".to_owned()));
    blocks.push(commands(&["gh auth login"]));
    blocks
}

fn commands(lines: &[&str]) -> Block {
    Block::Commands(lines.iter().map(|line| (*line).to_owned()).collect())
}

fn linux_install_command() -> Option<&'static str> {
    let (id, id_like) = linux_release()?;
    let in_family = |name: &str| id == name || id_like.iter().any(|l| l == name);

    let command = if in_family("arch") {
        "sudo pacman -S github-cli"
    } else if in_family("debian") || in_family("ubuntu") {
        DEBIAN_INSTALL
    } else if id == "fedora" {
        "sudo dnf install gh"
    } else if in_family("rhel") || in_family("fedora") || id == "centos" {
        "sudo dnf config-manager --add-repo https://cli.github.com/packages/rpm/gh-cli.repo\nsudo dnf install gh"
    } else if in_family("suse") || id.starts_with("opensuse") {
        "sudo zypper install gh"
    } else if id == "alpine" {
        "sudo apk add github-cli"
    } else if id == "void" {
        "sudo xbps-install -S github-cli"
    } else if in_family("gentoo") {
        "sudo emerge dev-util/github-cli"
    } else {
        return None;
    };

    Some(command)
}

fn linux_release() -> Option<(String, Vec<String>)> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    let mut id = None;
    let mut id_like = Vec::new();

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            id = Some(unquote(value));
        } else if let Some(value) = line.strip_prefix("ID_LIKE=") {
            id_like = unquote(value)
                .split_whitespace()
                .map(str::to_owned)
                .collect();
        }
    }

    id.map(|id| (id, id_like))
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

const DEBIAN_INSTALL: &str = "sudo mkdir -p -m 755 /etc/apt/keyrings
wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null
echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main\" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh";
