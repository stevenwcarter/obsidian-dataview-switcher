use async_walkdir::{Filtering, WalkDir};
use clap::Parser;
use futures_lite::stream::StreamExt;
use std::fs::{self, read_to_string};

use regex::Regex;

use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(short, long)]
    dryrun: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut entries = WalkDir::new(args.path).filter(|entry| async move {
        if let Some(true) = entry
            .path()
            .file_name()
            .map(|f| !f.to_string_lossy().ends_with(".md"))
        {
            return Filtering::Ignore;
        }
        if let Some(true) = entry
            .path()
            .file_name()
            .map(|f| f.to_string_lossy().starts_with('.'))
        {
            return Filtering::IgnoreDir;
        }
        let file = fs::metadata(entry.path());
        if let Ok(metadata) = file {
            if metadata.is_dir() {
                return Filtering::Ignore;
            }
        }
        Filtering::Continue
    });
    loop {
        match entries.next().await {
            Some(Ok(entry)) => {
                let filename = entry.path().display().to_string();

                println!("{filename}");

                let input = read_to_string(&filename).context("Could not read input")?;

                if let Some(data) = transform_dataview_queries(&input) {
                    match args.dryrun {
                        Some(true) => {
                            println!("\n\n\n{filename}\n\n{data}");
                        }
                        _ => {
                            fs::rename(&filename, format!("{}.orig", &filename))
                                .context("Could not move file")?;
                            fs::write(&filename, data).context("Could not write updated file")?;
                        }
                    }
                }
            }
            Some(Err(e)) => {
                eprintln!("error: {}", e);
                break;
            }
            None => break,
        }
    }

    Ok(())
}

fn transform_dataview_queries(input: &str) -> Option<String> {
    // Regex to capture dataview queries
    let re = Regex::new(r"(?ms)```dataview(.*?)```").unwrap();

    let mut updated = false;

    // Replace the matched queries with the desired HTML comment format
    let result = re.replace_all(input, |caps: &regex::Captures| {
        updated = true;
        let query = caps.get(1).unwrap().as_str().replace("\n", "");
        format!(
            "<!-- QueryToSerialize: {} -->",
            trim_whitespace(query.trim())
        )
    });

    if !updated {
        return None;
    }

    Some(result.to_string())
}

fn trim_whitespace(s: &str) -> String {
    // second attempt: only allocate a string
    let mut result = String::with_capacity(s.len());
    s.split_whitespace().for_each(|w| {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(w);
    });
    result
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn it_parses() {
        let result = transform_dataview_queries(
            "
        Test

        ```dataview
            FOR test
                FROM wherever
                WHERE something
        ```

        Test2",
        )
        .unwrap();

        println!("RESULT: {result}");

        assert_eq!(
            result,
            "
        Test

        <!-- QueryToSerialize: FOR test FROM wherever WHERE something -->

        Test2"
                .to_string()
        );
    }

    #[test]
    fn it_parses_multiple_in_one_file() {
        let result = transform_dataview_queries(
            "
        Test

        ```dataview
            FOR test
                FROM wherever
                WHERE something
        ```

        Test-between

        ```dataview
            FOR another
                FROM test
                WHERE hello
        ```

        Test2",
        )
        .unwrap();

        println!("RESULT: {result}");

        assert_eq!(
            result,
            "
        Test

        <!-- QueryToSerialize: FOR test FROM wherever WHERE something -->

        Test-between

        <!-- QueryToSerialize: FOR another FROM test WHERE hello -->

        Test2"
                .to_string()
        );
    }
}
