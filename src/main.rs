use clap::Parser;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::env;
use std::fs::{rename, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(version, about = "Sync Atuin history into Fish shell history")]
struct Cli {
    /// Path to the Atuin SQLite history database
    #[arg(long, value_name = "PATH", default_value_os_t = default_atuin_db_path())]
    atuin_db: PathBuf,

    /// Path to the Fish shell history file
    #[arg(long, value_name = "PATH", default_value_os_t = default_fish_history_path())]
    fish_history: PathBuf,

    /// Suppress timing output
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug)]
struct Entry {
    command: String,
    timestamp: i64,
    paths: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let total_start = Instant::now();

    let home_start = Instant::now();
    let _ = env::var("HOME")?;
    print_timing(cli.quiet, "Determining home directory", home_start);

    let db_start = Instant::now();
    let db_url = format!("sqlite://{}", cli.atuin_db.display());
    let pool = SqlitePoolOptions::new().max_connections(1).connect(&db_url).await?;
    print_timing(cli.quiet, "Connecting to SQLite database", db_start);

    let query_start = Instant::now();
    let rows = sqlx::query!("SELECT timestamp, command, cwd FROM history ORDER BY timestamp ASC")
        .fetch_all(&pool)
        .await?;
    print_timing(cli.quiet, "Reading Atuin history", query_start);

    let mut entries_map: HashMap<String, Entry> = HashMap::new();

    for row in rows {
        let timestamp = row.timestamp / 1_000_000_000;
        let cwd = if row.cwd != "unknown" && !row.cwd.is_empty() {
            Some(row.cwd)
        } else {
            None
        };

        entries_map
            .entry(row.command.clone())
            .and_modify(|e| {
                if timestamp > e.timestamp {
                    e.timestamp = timestamp;
                }
                if let Some(path) = &cwd {
                    if !e.paths.contains(path) {
                        e.paths.push(path.clone());
                    }
                }
            })
            .or_insert_with(|| Entry {
                command: row.command,
                timestamp,
                paths: cwd.into_iter().collect(),
            });
    }

    let mut entries: Vec<Entry> = entries_map.into_values().collect();
    entries.sort_by_key(|e| e.timestamp);

    let write_start = Instant::now();
    write_fish_history(&entries, &cli.fish_history)?;

    print_timing(cli.quiet, "Writing Fish history", write_start);

    print_timing(cli.quiet, "Total execution time", total_start);
    Ok(())
}

fn print_timing(quiet: bool, label: &str, start: Instant) {
    if !quiet {
        println!("{label} took: {:.3}s", start.elapsed().as_secs_f64());
    }
}

fn default_atuin_db_path() -> PathBuf {
    PathBuf::from(format!("{}/.local/share/atuin/history.db", home_dir()))
}

fn default_fish_history_path() -> PathBuf {
    PathBuf::from(format!("{}/.local/share/fish/fish_history", home_dir()))
}

fn home_dir() -> String {
    env::var("HOME").expect("HOME must be set")
}

fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
}

fn write_fish_history(entries: &[Entry], path: &PathBuf) -> std::io::Result<()> {
    let tmp_path = path.with_extension("tmp");

    {
        let file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let command = escape_yaml(&entry.command);
            writeln!(writer, "- cmd: {}", command)?;
            writeln!(writer, "  when: {}", entry.timestamp)?;

            if !entry.paths.is_empty() {
                writeln!(writer, "  paths:")?;
                for path in &entry.paths {
                    let path = escape_yaml(path);
                    writeln!(writer, "    - {}", path)?;
                }
            }
        }

        writer.flush()?;               // flush BufWriter
        writer.get_ref().sync_all()?;  // fsync file
    }

    rename(tmp_path, path)?; // atomic replace on Unix

    Ok(())
}
