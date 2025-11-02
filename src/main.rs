use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug)]
struct Entry {
    command: String,
    timestamp: i64,
    paths: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let total_start = Instant::now();

    let home_start = Instant::now();
    let home_dir = env::var("HOME")?;
    println!("Determining home directory took: {:.3}s", home_start.elapsed().as_secs_f64());

    let db_start = Instant::now();
    let db_path = format!("{}/.local/share/atuin/history.db", home_dir);
    let db_url = format!("sqlite://{}", db_path);
    let pool = SqlitePoolOptions::new().max_connections(1).connect(&db_url).await?;
    println!("Connecting to SQLite database took: {:.3}s", db_start.elapsed().as_secs_f64());

    let query_start = Instant::now();
    let rows = sqlx::query!("SELECT timestamp, command, cwd FROM history ORDER BY timestamp ASC")
        .fetch_all(&pool)
        .await?;
    println!("Reading Atuin history took: {:.3}s", query_start.elapsed().as_secs_f64());

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
    let fish_history_path = PathBuf::from(format!("{}/.local/share/fish/fish_history", home_dir));
    write_fish_history(&entries, &fish_history_path)?;

    println!("Writing Fish history took: {:.3}s", write_start.elapsed().as_secs_f64());

    println!("Total execution time: {:.3}s", total_start.elapsed().as_secs_f64());
    Ok(())
}

fn write_fish_history(entries: &[Entry], path: &PathBuf) -> std::io::Result<()> {
    let mut content = String::new();
    for entry in entries {
        let command = entry.command.replace("\n", "\\n");
        content.push_str(&format!("- cmd: {}\n  when: {}\n", command, entry.timestamp));
        if !entry.paths.is_empty() {
            content.push_str("  paths:\n");
            for path in &entry.paths {
                content.push_str(&format!("    - {}\n", path));
            }
        }
    }
    std::fs::write(path, content)?;
    Ok(())
}
