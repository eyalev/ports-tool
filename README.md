# ports-tool

A Rust CLI tool to display open ports with detailed process information.

## Features

- 🔍 **Comprehensive Port Information**: Shows TCP/UDP ports with PID, process name, full command, and working directory
- 📋 **Multiple Display Formats**: 
  - Standard (default): Clean table with truncated text
  - Compact (`-c`): Solid Unicode borders with text wrapping
  - Detailed (`-d`): Individual sections per port
- 🎯 **Advanced Filtering**:
  - Include filter (`-f`): Show only entries containing specified text
  - Exclude filter (`-x`): Hide entries containing specified text
  - Combine both for precise control
- 🏠 **Localhost Focus**: Default localhost-only view with option to show all ports
- 🎯 **Specific Port Lookup**: Target individual ports for detailed inspection

## Installation

### Quick Install (One Command - No sudo required)
```bash
curl -sSL https://raw.githubusercontent.com/eyalev/ports-tool/master/install.sh | bash
```
*Installs to `~/.local/bin` with automatic shell detection and PATH setup*

### Manual Install from GitHub
```bash
# Download and install latest release to ~/.local/bin
mkdir -p ~/.local/bin
curl -L https://github.com/eyalev/ports-tool/releases/latest/download/ports-tool -o ~/.local/bin/ports-tool
chmod +x ~/.local/bin/ports-tool

# Add to PATH if needed
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc
```

### System-wide Install (requires sudo)
```bash
# Download and install system-wide
curl -L https://github.com/eyalev/ports-tool/releases/latest/download/ports-tool -o ports-tool
chmod +x ports-tool
sudo mv ports-tool /usr/local/bin/
```

### From Source
```bash
git clone https://github.com/eyalev/ports-tool.git
cd ports-tool
cargo build --release
cp target/release/ports-tool ~/.local/bin/  # User install
# OR
sudo cp target/release/ports-tool /usr/local/bin/  # System install
```

## Uninstall

### Quick Uninstall
```bash
curl -sSL https://raw.githubusercontent.com/eyalev/ports-tool/master/install.sh | bash -s uninstall
```

### Manual Uninstall
```bash
# Remove from user installation
rm ~/.local/bin/ports-tool

# Remove from system installation (if installed with sudo)
sudo rm /usr/local/bin/ports-tool
```

The uninstall script will:
- ✅ Remove `ports-tool` from `~/.local/bin`
- 🔍 Check for system-wide installations and provide removal instructions
- 🧹 Clean removal with confirmation

## Usage

```bash
# Show all localhost ports (default)
ports-tool

# Compact format with solid lines and text wrapping
ports-tool -c

# Detailed format with full information
ports-tool -d

# Filter for personal projects only
ports-tool -f personal

# Exclude VS Code processes
ports-tool -x code

# Show personal projects but hide Chrome processes
ports-tool -f personal -x chrome

# Check specific port
ports-tool -p 8080

# Show all ports (not just localhost)
ports-tool -a

# Combine options for powerful filtering
ports-tool -c -f node -x chrome

# Uninstall ports-tool
curl -sSL https://raw.githubusercontent.com/eyalev/ports-tool/master/install.sh | bash -s uninstall
```

## Command Line Options

```
Options:
  -l, --localhost       Show only localhost ports (default)
  -a, --all             Show all ports (including non-localhost)  
  -p, --port <PORT>     Check specific port
  -d, --detailed        Show detailed output with full paths and commands
  -c, --compact         Show narrow format for small terminals
  -f, --filter <TEXT>   Filter results by text (include)
  -x, --exclude <TEXT>  Exclude results containing text
  -n, --limit <COUNT>   Limit the number of results displayed
  -h, --help            Print help
  -V, --version         Print version
```

## Examples

### Basic Usage
```bash
# Quick overview of active development servers
ports-tool -c -f node

# Find what's running on port 3000
ports-tool -p 3000 -d

# Clean view without editor processes
ports-tool -c -x code -x chrome

# Show only the first 5 open ports
ports-tool -n 5

# Get top 3 node processes
ports-tool -f node -n 3
```

### Output Formats

#### Standard Format
```
+-------+----------+--------+---------+---------+--------------------------------+--------------------------------+
| PORT  | PROTOCOL | STATE  | PID     | PROCESS | COMMAND                        | WORKING_DIR                    |
+-------+----------+--------+---------+---------+--------------------------------+--------------------------------+
| 3000  | TCP      | LISTEN | 1234    | node    | node server.js                 | /home/user/my-project          |
+-------+----------+--------+---------+---------+--------------------------------+--------------------------------+
```

#### Compact Format (`-c`)
```
┌───────┬──────────┬────────┬─────────┬─────────┬────────────────────────────────┐
│ PORT  │ PROTOCOL │ STATE  │ PID     │ PROCESS │ COMMAND                        │
├───────┼──────────┼────────┼─────────┼─────────┼────────────────────────────────┤
│ 3000  │ TCP      │ LISTEN │ 1234    │ node    │ node /long/path/to/server.js   │
│       │          │        │         │         │ --port 3000 --dev             │
└───────┴──────────┴────────┴─────────┴─────────┴────────────────────────────────┘
```

#### Detailed Format (`-d`)
```
Port: 3000 (TCP)
State: LISTEN
PID: 1234
Process: node
Command: node /home/user/my-project/server.js --port 3000 --dev
Working Dir: /home/user/my-project
```

## How It Works

`ports-tool` reads from `/proc/net/tcp` and `/proc/net/udp` to discover open ports, then matches socket inodes to processes in `/proc/*/fd/` to gather detailed process information including:

- Process ID and name
- Complete command line with arguments
- Working directory when the process started
- Port state and protocol information

## Requirements

- Linux system with `/proc` filesystem
- Rust 1.70+ (for building from source)

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions welcome! Please feel free to submit issues and pull requests.