use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

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

    let mut entries: Vec<String> = Vec::new();
    for row in rows {
        let mut line = format!("- cmd: {}\n  when: {}\n", row.command, row.timestamp / 1_000_000_000);

         if &row.cwd != "unknown" {
            if !row.cwd.is_empty() {
                line.push_str(&format!("  paths:\n    - {}\n", row.cwd));
            }
        }

        entries.push(line);
    }

    let write_start = Instant::now();
    let fish_history_path = PathBuf::from(format!("{}/.local/share/fish/fish_history", home_dir));
    write_fish_history(&entries, &fish_history_path)?;
    println!("Writing Fish history took: {:.3}s", write_start.elapsed().as_secs_f64());

    println!("Total execution time: {:.3}s", total_start.elapsed().as_secs_f64());
    Ok(())
}

fn write_fish_history(entries: &[String], path: &PathBuf) -> std::io::Result<()> {
    let content = entries.concat();
    std::fs::write(path, content)?;
    Ok(())
}
