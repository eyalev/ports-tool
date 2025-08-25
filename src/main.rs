use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use clap::{Arg, Command as ClapCommand};
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled, settings::{Style, Width, object::Columns}};

#[derive(Debug, Serialize, Deserialize, Tabled)]
struct PortInfo {
    #[tabled(rename = "PORT")]
    port: u16,
    #[tabled(rename = "PROTOCOL")]
    protocol: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "PROCESS")]
    process_name: String,
    #[tabled(rename = "COMMAND")]
    command: String,
    #[tabled(rename = "WORKING_DIR")]
    working_dir: String,
}

#[derive(Debug)]
struct ProcessInfo {
    pid: u32,
    name: String,
    command: String,
    working_dir: String,
}

fn main() -> Result<()> {
    let matches = ClapCommand::new("ports-tool")
        .version("0.1.0")
        .author("Port Scanner Tool")
        .about("Shows open ports with process information")
        .arg(
            Arg::new("localhost")
                .short('l')
                .long("localhost")
                .help("Show only localhost ports")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .help("Show all ports (including non-localhost)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Check specific port")
                .value_name("PORT"),
        )
        .arg(
            Arg::new("detailed")
                .short('d')
                .long("detailed")
                .help("Show detailed output with full paths and commands")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("compact")
                .short('c')
                .long("compact")
                .help("Show compact table format")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("filter")
                .help("Filter results by text (searches in process name, command, and working directory)")
                .value_name("TEXT"),
        )
        .arg(
            Arg::new("exclude")
                .short('x')
                .long("exclude")
                .help("Exclude results containing text (searches in process name, command, and working directory)")
                .value_name("TEXT"),
        )
        .get_matches();

    let localhost_only = matches.get_flag("localhost") || !matches.get_flag("all");
    let specific_port: Option<u16> = matches
        .get_one::<String>("port")
        .and_then(|p| p.parse().ok());
    let detailed = matches.get_flag("detailed");
    let compact = matches.get_flag("compact");
    let filter_text = matches.get_one::<String>("filter");
    let exclude_text = matches.get_one::<String>("exclude");

    let mut ports = get_open_ports(localhost_only, specific_port)?;
    
    // Apply include filter if specified
    if let Some(filter) = filter_text {
        ports = filter_ports(ports, filter);
    }
    
    // Apply exclude filter if specified
    if let Some(exclude) = exclude_text {
        ports = exclude_ports(ports, exclude);
    }
    
    display_ports(&ports, detailed, compact)?;

    Ok(())
}

fn get_open_ports(localhost_only: bool, specific_port: Option<u16>) -> Result<Vec<PortInfo>> {
    let mut ports = Vec::new();
    let process_map = get_process_info_map()?;

    // Parse /proc/net/tcp for IPv4 TCP connections
    if let Ok(tcp_content) = fs::read_to_string("/proc/net/tcp") {
        for line in tcp_content.lines().skip(1) {
            if let Some(port_info) = parse_net_line(line, "tcp", &process_map, localhost_only, specific_port)? {
                ports.push(port_info);
            }
        }
    }

    // Parse /proc/net/udp for IPv4 UDP connections
    if let Ok(udp_content) = fs::read_to_string("/proc/net/udp") {
        for line in udp_content.lines().skip(1) {
            if let Some(port_info) = parse_net_line(line, "udp", &process_map, localhost_only, specific_port)? {
                ports.push(port_info);
            }
        }
    }

    ports.sort_by_key(|p| p.port);
    Ok(ports)
}

fn parse_net_line(
    line: &str,
    protocol: &str,
    process_map: &HashMap<u32, ProcessInfo>,
    localhost_only: bool,
    specific_port: Option<u16>,
) -> Result<Option<PortInfo>> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return Ok(None);
    }

    let local_address = fields[1];
    let state = fields[3];

    // Parse address:port
    let addr_parts: Vec<&str> = local_address.split(':').collect();
    if addr_parts.len() != 2 {
        return Ok(None);
    }

    let port = u16::from_str_radix(addr_parts[1], 16).unwrap_or(0);
    let addr = u32::from_str_radix(addr_parts[0], 16).unwrap_or(0);

    // Convert to IP address (little-endian)
    let ip = format!(
        "{}.{}.{}.{}",
        addr & 0xFF,
        (addr >> 8) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 24) & 0xFF
    );

    // Filter for localhost if requested
    if localhost_only && ip != "127.0.0.1" && ip != "0.0.0.0" {
        return Ok(None);
    }

    // Filter for specific port if requested
    if let Some(target_port) = specific_port {
        if port != target_port {
            return Ok(None);
        }
    }

    // For TCP, only show listening ports (state 0A = LISTEN)
    let state_str = if protocol == "tcp" {
        match state {
            "0A" => "LISTEN",
            "01" => "ESTABLISHED",
            "02" => "SYN_SENT",
            "03" => "SYN_RECV",
            "04" => "FIN_WAIT1",
            "05" => "FIN_WAIT2",
            "06" => "TIME_WAIT",
            "07" => "CLOSE",
            "08" => "CLOSE_WAIT",
            "09" => "LAST_ACK",
            "0B" => "CLOSING",
            _ => "UNKNOWN",
        }
    } else {
        "OPEN"
    };

    // For TCP, we mainly want listening ports
    if protocol == "tcp" && state != "0A" && specific_port.is_none() {
        return Ok(None);
    }

    // Try to get the inode to find the process
    let inode: u32 = fields.get(9).unwrap_or(&"0").parse().unwrap_or(0);
    let (pid, process_info) = if inode > 0 {
        find_process_by_inode(inode, process_map)?
    } else {
        (None, None)
    };

    Ok(Some(PortInfo {
        port,
        protocol: protocol.to_uppercase(),
        state: state_str.to_string(),
        pid: pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
        process_name: process_info.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "-".to_string()),
        command: process_info.as_ref().map(|p| p.command.clone()).unwrap_or_else(|| "-".to_string()),
        working_dir: process_info.as_ref().map(|p| p.working_dir.clone()).unwrap_or_else(|| "-".to_string()),
    }))
}

fn get_process_info_map() -> Result<HashMap<u32, ProcessInfo>> {
    let mut process_map = HashMap::new();

    if let Ok(proc_dir) = fs::read_dir("/proc") {
        for entry in proc_dir.flatten() {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                if let Ok(process_info) = get_process_info(pid) {
                    process_map.insert(pid, process_info);
                }
            }
        }
    }

    Ok(process_map)
}

fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    let comm_path = format!("/proc/{}/comm", pid);
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let cwd_path = format!("/proc/{}/cwd", pid);

    let name = fs::read_to_string(&comm_path)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let command = fs::read_to_string(&cmdline_path)
        .unwrap_or_else(|_| "unknown".to_string())
        .replace('\0', " ")
        .trim()
        .to_string();

    let working_dir = fs::read_link(&cwd_path)
        .unwrap_or_else(|_| std::path::PathBuf::from("unknown"))
        .to_string_lossy()
        .to_string();

    Ok(ProcessInfo {
        pid,
        name: name.clone(),
        command: if command.is_empty() {
            name
        } else {
            command
        },
        working_dir,
    })
}

fn find_process_by_inode(
    target_inode: u32,
    process_map: &HashMap<u32, ProcessInfo>,
) -> Result<(Option<u32>, Option<ProcessInfo>)> {
    for (pid, process_info) in process_map {
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(entries) = fs::read_dir(&fd_dir) {
            for entry in entries.flatten() {
                if let Ok(link_target) = fs::read_link(entry.path()) {
                    if let Some(target_str) = link_target.to_str() {
                        if target_str.starts_with("socket:[") {
                            if let Some(inode_str) = target_str
                                .strip_prefix("socket:[")
                                .and_then(|s| s.strip_suffix(']'))
                            {
                                if let Ok(inode) = inode_str.parse::<u32>() {
                                    if inode == target_inode {
                                        return Ok((Some(*pid), Some(process_info.clone())));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok((None, None))
}

fn filter_ports(ports: Vec<PortInfo>, filter_text: &str) -> Vec<PortInfo> {
    let filter_lower = filter_text.to_lowercase();
    ports.into_iter().filter(|port| {
        port.process_name.to_lowercase().contains(&filter_lower) ||
        port.command.to_lowercase().contains(&filter_lower) ||
        port.working_dir.to_lowercase().contains(&filter_lower)
    }).collect()
}

fn exclude_ports(ports: Vec<PortInfo>, exclude_text: &str) -> Vec<PortInfo> {
    let exclude_lower = exclude_text.to_lowercase();
    ports.into_iter().filter(|port| {
        !port.process_name.to_lowercase().contains(&exclude_lower) &&
        !port.command.to_lowercase().contains(&exclude_lower) &&
        !port.working_dir.to_lowercase().contains(&exclude_lower)
    }).collect()
}

fn display_ports(ports: &[PortInfo], detailed: bool, compact: bool) -> Result<()> {
    if ports.is_empty() {
        println!("No open ports found.");
        return Ok(());
    }

    if detailed {
        display_detailed_format(ports)
    } else if compact {
        display_compact_table(ports)
    } else {
        display_standard_table(ports)
    }
}

fn display_detailed_format(ports: &[PortInfo]) -> Result<()> {
    for (i, port) in ports.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("Port: {} ({})", port.port, port.protocol);
        println!("State: {}", port.state);
        println!("PID: {}", port.pid);
        println!("Process: {}", port.process_name);
        println!("Command: {}", port.command);
        println!("Working Dir: {}", port.working_dir);
        println!("{}", "-".repeat(60));
    }
    Ok(())
}

fn display_compact_table(ports: &[PortInfo]) -> Result<()> {
    let mut table = Table::new(ports);
    table.with(Style::modern());
    table.modify(Columns::new(5..), Width::wrap(50).keep_words(true));
    println!("{}", table);
    Ok(())
}

fn display_standard_table(ports: &[PortInfo]) -> Result<()> {
    let ports_truncated: Vec<PortInfo> = ports.iter().map(|p| PortInfo {
        port: p.port,
        protocol: p.protocol.clone(),
        state: p.state.clone(),
        pid: p.pid.clone(),
        process_name: p.process_name.clone(),
        command: truncate_string(&p.command, 30),
        working_dir: truncate_string(&p.working_dir, 30),
    }).collect();

    let mut table = Table::new(&ports_truncated);
    table.with(Style::ascii());
    println!("{}", table);
    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

impl Clone for ProcessInfo {
    fn clone(&self) -> Self {
        ProcessInfo {
            pid: self.pid,
            name: self.name.clone(),
            command: self.command.clone(),
            working_dir: self.working_dir.clone(),
        }
    }
}