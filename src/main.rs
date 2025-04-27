use std::fs;
use std::io;
use std::path::PathBuf;
use sysinfo::CpuExt;
use machine_info::Machine;
use clap::{Parser, Subcommand};
use colored::Colorize; // Fixed import for colorize trait
use dirs::home_dir;
use sysinfo::{System, SystemExt, ComponentExt};
use toml::{Table, Value};
use users::{get_current_uid, get_user_by_uid};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Import custom ASCII art from file
    ImportArt {
        /// Path to the art file
        #[arg(required = true)]
        file: String,
    },
}

struct Config {
    display: DisplayConfig,
    ascii: AsciiConfig,
    format: FormatConfig,
}

struct DisplayConfig {
    show_username: bool,
    show_host: bool,
    show_os: bool,
    show_kernel: bool,
    show_uptime: bool,
    show_cpu: bool,
    show_memory: bool,
    show_gpu: bool,
    show_shell: bool,
    show_rust: bool,
}

struct AsciiConfig {
    art_style: String,
    color: String,
    art_width: usize,
    custom_art: Option<String>,
    custom_art_path: Option<String>,
}

struct FormatConfig {
    info_prefix: String,
    info_color: String,
    label_color: String,
    title_color: String,
    max_width: usize,
}

fn colorize(text: &str, color_name: &str) -> colored::ColoredString {
    match color_name.to_lowercase().as_str() {
        "red" => text.red(),
        "green" => text.green(),
        "yellow" => text.yellow(),
        "blue" => text.blue(),
        "magenta" => text.magenta(),
        "cyan" => text.cyan(),
        "white" => text.white(),
        "black" => text.black(),
        _ => text.normal(),
    }
}

fn load_config() -> Config {
    let default_config = Config {
        display: DisplayConfig {
            show_username: true,
            show_host: true,
            show_os: true,
            show_kernel: true,
            show_uptime: true,
            show_cpu: true,
            show_memory: true,
            show_gpu: true,
            show_shell: true,
            show_rust: true,
        },
        ascii: AsciiConfig {
            art_style: "crab".to_string(),
            color: "red".to_string(),
            art_width: 16,
            custom_art: None,
            custom_art_path: None,
        },
        format: FormatConfig {
            info_prefix: "".to_string(),
            info_color: "white".to_string(),
            label_color: "yellow".to_string(),
            title_color: "red".to_string(),
            max_width: 60,
        },
    };

    let mut config_paths = Vec::new();
    
    if let Some(home) = home_dir() {
        config_paths.push(home.join(".config").join("rsfetch").join("config.toml"));
    }
    
    config_paths.push(PathBuf::from("/etc/rsfetch/config.toml"));
    
    // Current directory config
    config_paths.push(PathBuf::from("rsfetch_config.toml"));

    for path in config_paths {
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(parsed_toml) = content.parse::<Table>() {
                        return parse_config(parsed_toml, &default_config);
                    }
                }
                Err(e) => eprintln!("Error reading config file {}: {}", path.display(), e),
            }
        }
    }

    default_config
}

fn parse_config(toml_table: Table, default_config: &Config) -> Config {
    let mut config = Config {
        display: DisplayConfig {
            show_username: default_config.display.show_username,
            show_host: default_config.display.show_host,
            show_os: default_config.display.show_os,
            show_kernel: default_config.display.show_kernel,
            show_uptime: default_config.display.show_uptime,
            show_cpu: default_config.display.show_cpu,
            show_memory: default_config.display.show_memory,
            show_gpu: default_config.display.show_gpu,
            show_shell: default_config.display.show_shell,
            show_rust: default_config.display.show_rust,
        },
        ascii: AsciiConfig {
            art_style: default_config.ascii.art_style.clone(),
            color: default_config.ascii.color.clone(),
            art_width: default_config.ascii.art_width,
            custom_art: default_config.ascii.custom_art.clone(),
            custom_art_path: default_config.ascii.custom_art_path.clone(),
        },
        format: FormatConfig {
            info_prefix: default_config.format.info_prefix.clone(),
            info_color: default_config.format.info_color.clone(),
            label_color: default_config.format.label_color.clone(),
            title_color: default_config.format.title_color.clone(),
            max_width: default_config.format.max_width,
        },
    };

    if let Some(display) = toml_table.get("display").and_then(|v| v.as_table()) {
        config.display.show_username = display.get("show_username").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_username);
        config.display.show_host = display.get("show_host").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_host);
        config.display.show_os = display.get("show_os").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_os);
        config.display.show_kernel = display.get("show_kernel").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_kernel);
        config.display.show_uptime = display.get("show_uptime").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_uptime);
        config.display.show_cpu = display.get("show_cpu").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_cpu);
        config.display.show_memory = display.get("show_memory").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_memory);
        config.display.show_gpu = display.get("show_gpu").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_gpu);
        config.display.show_shell = display.get("show_shell").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_shell);
        config.display.show_rust = display.get("show_rust").and_then(|v| v.as_bool()).unwrap_or(default_config.display.show_rust);
    }

    if let Some(ascii) = toml_table.get("ascii").and_then(|v| v.as_table()) {
        config.ascii.art_style = ascii.get("art_style").and_then(|v| v.as_str()).unwrap_or(&default_config.ascii.art_style).to_string();
        config.ascii.color = ascii.get("color").and_then(|v| v.as_str()).unwrap_or(&default_config.ascii.color).to_string();
        config.ascii.art_width = ascii.get("art_width").and_then(|v| v.as_integer()).unwrap_or(default_config.ascii.art_width as i64) as usize;
        config.ascii.custom_art = ascii.get("custom_art").and_then(|v| v.as_str()).map(|s| s.to_string());
        config.ascii.custom_art_path = ascii.get("custom_art_path").and_then(|v| v.as_str()).map(|s| s.to_string());
    }

    if let Some(format) = toml_table.get("format").and_then(|v| v.as_table()) {
        config.format.info_prefix = format.get("info_prefix").and_then(|v| v.as_str()).unwrap_or(&default_config.format.info_prefix).to_string();
        config.format.info_color = format.get("info_color").and_then(|v| v.as_str()).unwrap_or(&default_config.format.info_color).to_string();
        config.format.label_color = format.get("label_color").and_then(|v| v.as_str()).unwrap_or(&default_config.format.label_color).to_string();
        config.format.title_color = format.get("title_color").and_then(|v| v.as_str()).unwrap_or(&default_config.format.title_color).to_string();
        config.format.max_width = format.get("max_width").and_then(|v| v.as_integer()).unwrap_or(default_config.format.max_width as i64) as usize;
    }

    config
}

fn get_ascii_art(config: &Config) -> Vec<String> {
    if let Some(ref custom_art) = config.ascii.custom_art {
        if !custom_art.is_empty() {
            return custom_art.lines().map(|s| s.to_string()).collect();
        }
    }

    if let Some(ref path) = config.ascii.custom_art_path {
        if !path.is_empty() {
            let expanded_path = shellexpand::tilde(path);
            if let Ok(content) = fs::read_to_string(expanded_path.as_ref()) {
                return content.lines().map(|s| s.to_string()).collect();
            } else {
                eprintln!("Error loading custom art from {}", path);
            }
        }
    }

    match config.ascii.art_style.as_str() {
        "cat" => {
            r#"
  /\_/\  
 ( o.o ) 
  > ^ <  
 /  _  \ 
(____|____)"#
        },
        "crab" => {
            r#"
    _~^~^~_
\) /  o o  \ (/
  '_   V   _'
  / '-----' \"#
        },
        "ferris" => {
            r#"
     _~^~^~_
 \) /  o o  \ (/
   '_   -   _'
   / '-----' \
             "#
        },
        "rust" => {
            r#"
                     
      ____     
     /\  _`\   
   __\ \ \L\ \  
 /'__`\ \  _ <' 
/\  __/\ \ \L\ \
\ \____\\ \____/
 \/____/ \/___/ "#
        },
        "tux" => {
            r#"
   .--.   
  |o_o |  
  |:_/ |  
 //   \ \ 
(|     | )
/'\_   _/`\
\___)=(___/"#
        },
        "rsfetch" => {
            r#"
     ----    
    |*   |_____
  __|___       |
 |       ______|
 |_____|   |   
       |  *|
        ----"#
        },
        "tree" => {
            r#"
       /\\  
      /**\\  
     /****\\  
    /******\\  
   /********\\  
  /__________\\  
       ||  
       ||"#
        },
        _ => {
            r#"
    _~^~^~_
\) /  o o  \ (/
  '_   V   _'
  / '-----' \"#
        }
    }.lines().map(|s| s.to_string()).collect()
}

fn get_username() -> String {
    match get_user_by_uid(get_current_uid()) {
        Some(user) => user.name().to_string_lossy().to_string(),
        None => whoami::username(),
    }
}

fn get_kernel() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    sys.kernel_version().unwrap_or_else(|| "Unknown".to_string())
}

fn get_os_info() -> String {
    whoami::distro()
}

fn get_uptime() -> String {
    let sys = System::new_all();
    let uptime_seconds = sys.uptime();
    
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else {
        format!("{}h {}m", hours, minutes)
    }
}

fn get_cpu_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let processors = sys.cpus();
    
    if let Some(processor) = processors.first() {
        let mut cpu_name = processor.brand().to_string();
        cpu_name = cpu_name.replace("(R)", "");
        cpu_name = cpu_name.replace("(TM)", "");
        cpu_name = cpu_name.replace("CPU", "");
        cpu_name.trim().to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_memory_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0; // Convert to GB
    let used_memory = sys.total_memory() - sys.available_memory();
    let used_memory_gb = used_memory as f64 / 1_073_741_824.0;
    let memory_percent = (used_memory as f64 / sys.total_memory() as f64) * 100.0;
    
    format!("{}% ({:.1}GB/{:.1}GB)", memory_percent as u64, used_memory_gb, total_memory_gb)
}

fn get_gpu_info() -> String {
    let machine = Machine::new();                // ⬅️ no Result, just returns Machine
    if let Some(gpu) = machine.gpu.first() {
        let brand = gpu.vendor.clone().unwrap_or_default();
        let model = gpu.name.clone().unwrap_or_default();
        return format!("{} {}", brand, model);
    }
    "Unknown GPU".to_string()
}

fn get_shell() -> String {
    match std::env::var("SHELL") {
        Ok(shell) => {
            shell.split('/').last().unwrap_or("unknown").to_string()
        },
        Err(_) => "unknown".to_string()
    }
}

fn get_rust_info() -> String {
    match std::process::Command::new("rustc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        },
        _ => "Rust not found".to_string()
    }
}

fn print_title(config: &Config) {
    let title = "RsFetch - System Information";
    let centered_title = format!("{:^width$}", title, width = config.format.max_width);
    
    println!("\n{}\n", colorize(&centered_title, &config.format.title_color));
    println!("{}", colorize(&"-".repeat(config.format.max_width), &config.format.title_color));
}

fn import_art(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let art_content = fs::read_to_string(file_path)?;
    
    let config_dir = home_dir().unwrap_or_default().join(".config").join("rsfetch");
    fs::create_dir_all(&config_dir)?;
    
    let config_path = config_dir.join("config.toml");
    
    let mut config_toml = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        content.parse::<Table>().unwrap_or_default()
    } else {
        Table::new()
    };
    
    if !config_toml.contains_key("ascii") {
        config_toml.insert("ascii".to_string(), Value::Table(Table::new()));
    }
    
    if let Some(ascii) = config_toml.get_mut("ascii").and_then(|v| v.as_table_mut()) {
        ascii.insert("art_style".to_string(), Value::String("custom".to_string()));
        ascii.insert("custom_art".to_string(), Value::String(art_content));
    }
    
    let toml_string = toml::to_string(&config_toml)?;
    fs::write(&config_path, toml_string)?;
    
    println!("Successfully imported ASCII art from {}", file_path);
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    if let Some(Commands::ImportArt { file }) = args.command {
        if let Err(e) = import_art(&file) {
            eprintln!("Error importing art: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    let config = load_config();
    
    print_title(&config);
    
    let mut info = Vec::new();
    
    if config.display.show_username {
        info.push(("Username".to_string(), get_username()));
    }
    
    info.push((
    "Host".to_string(),
    whoami::fallible::hostname().unwrap_or_else(|_| "Unknown".to_string())
    ));

    
    if config.display.show_os {
        info.push(("OS".to_string(), get_os_info()));
    }
    
    if config.display.show_kernel {
        info.push(("Kernel".to_string(), get_kernel()));
    }
    
    if config.display.show_uptime {
        info.push(("Uptime".to_string(), get_uptime()));
    }
    
    if config.display.show_cpu {
        info.push(("CPU".to_string(), get_cpu_info()));
    }
    
    if config.display.show_memory {
        info.push(("Memory".to_string(), get_memory_usage()));
    }
    
    if config.display.show_gpu {
        info.push(("GPU".to_string(), get_gpu_info()));
    }
    
    if config.display.show_shell {
        info.push(("Shell".to_string(), get_shell()));
    }
    
    if config.display.show_rust {
        info.push(("Rust".to_string(), get_rust_info()));
    }
    
    // Get and format ASCII art
    let ascii_art = get_ascii_art(&config);
    let art_width = config.ascii.art_width;
    
    // Display with perfect alignment
    for (i, (key, value)) in info.iter().enumerate() {
        let art_line = if i < ascii_art.len() {
            &ascii_art[i]
        } else {
            ""
        };
        
        let padding = std::cmp::max(art_width + 2, 11); // Minimum padding
        let padded_art = format!("{:<width$}", art_line, width = padding);
        
        println!(
            "{} {}{}: {}",
            padded_art,
            config.format.info_prefix,
            colorize(key, &config.format.label_color),
            colorize(&value, &config.format.info_color)
        );
    }
}
