use std::time::Duration;

use tokio::process::Command;
use tokio::time::sleep;

/// Parse the rate-limit wait duration from coderabbit output.
/// Expects a format like "Try after X minutes and Y seconds".
fn parse_rate_limit_wait(output: &str) -> Option<Duration> {
    let lower = output.to_lowercase();
    let marker = "try after ";
    let idx = lower.find(marker)?;
    let rest = &lower[idx + marker.len()..];

    let mut total_seconds = 0u64;
    let mut remaining = rest.trim();

    // Parse minutes
    if let Some(min_idx) = remaining.find(" minute")
        && let Ok(mins) = remaining[..min_idx].trim().parse::<u64>()
    {
        total_seconds += mins * 60;
        let after_min = &remaining[min_idx..];
        if let Some(and_idx) = after_min.find("and ") {
            remaining = &after_min[and_idx + "and ".len()..];
        } else {
            remaining = "";
        }
    }

    // Parse seconds
    if let Some(sec_idx) = remaining.find(" second")
        && let Ok(secs) = remaining[..sec_idx].trim().parse::<u64>()
    {
        total_seconds += secs;
    }

    if total_seconds > 0 {
        Some(Duration::from_secs(total_seconds))
    } else {
        None
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    loop {
        let output = match Command::new("coderabbit")
            .arg("--prompt-only")
            .args(&args)
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to run coderabbit: {e}");
                std::process::exit(1);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");

        if let Some(wait_duration) = parse_rate_limit_wait(&combined) {
            sleep(wait_duration).await;
            continue;
        }

        if !output.status.success() {
            eprintln!("{}", combined.trim());
            std::process::exit(output.status.code().unwrap_or(1));
        }

        print!("{stdout}");
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minutes_and_seconds() {
        let duration = parse_rate_limit_wait("Try after 2 minutes and 7 seconds").unwrap();
        assert_eq!(duration, Duration::from_secs(2 * 60 + 7));
    }

    #[test]
    fn test_parse_minutes_only() {
        let duration = parse_rate_limit_wait("Try after 3 minutes").unwrap();
        assert_eq!(duration, Duration::from_secs(3 * 60));
    }

    #[test]
    fn test_parse_seconds_only() {
        let duration = parse_rate_limit_wait("Try after 45 seconds").unwrap();
        assert_eq!(duration, Duration::from_secs(45));
    }

    #[test]
    fn test_parse_in_longer_message() {
        let output = "Rate limit exceeded. Try after 2 minutes and 7 seconds.";
        let duration = parse_rate_limit_wait(output).unwrap();
        assert_eq!(duration, Duration::from_secs(2 * 60 + 7));
    }

    #[test]
    fn test_parse_singular_minute() {
        let duration = parse_rate_limit_wait("Try after 1 minute and 30 seconds").unwrap();
        assert_eq!(duration, Duration::from_secs(90));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let duration = parse_rate_limit_wait("TRY AFTER 1 MINUTES AND 30 SECONDS").unwrap();
        assert_eq!(duration, Duration::from_secs(90));
    }

    #[test]
    fn test_parse_no_match() {
        assert!(parse_rate_limit_wait("Some other error message").is_none());
    }

    #[test]
    fn test_parse_zero_duration_returns_none() {
        assert!(parse_rate_limit_wait("Try after 0 minutes and 0 seconds").is_none());
    }

    #[test]
    fn test_parse_zero_minutes_nonzero_seconds() {
        let duration = parse_rate_limit_wait("Try after 0 minutes and 5 seconds").unwrap();
        assert_eq!(duration, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(parse_rate_limit_wait("").is_none());
    }

    #[test]
    fn test_parse_multiline_output() {
        let output = "Reviewing...\nError: Try after 1 minutes and 20 seconds\nPlease try again.";
        let duration = parse_rate_limit_wait(output).unwrap();
        assert_eq!(duration, Duration::from_secs(80));
    }
}
