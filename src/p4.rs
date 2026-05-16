use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct ZtagOutput {
    pub records: Vec<HashMap<String, String>>,
}

pub fn parse_ztag(output: &str) -> ZtagOutput {
    let mut records = Vec::new();
    let mut current_record = HashMap::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !current_record.is_empty() {
                records.push(current_record);
                current_record = HashMap::new();
            }
            continue;
        }

        if line.starts_with("... ") {
            let parts: Vec<&str> = line[4..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                current_record.insert(parts[0].to_string(), parts[1].to_string());
            } else if parts.len() == 1 {
                current_record.insert(parts[0].to_string(), "".to_string());
            }
        }
    }

    if !current_record.is_empty() {
        records.push(current_record);
    }

    ZtagOutput { records }
}

pub async fn run_p4(args: Vec<&str>) -> Result<String, String> {
    let output = Command::new("p4")
        .arg("-Ztag")
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ztag() {
        let output = r#"... depotFile //depot/file1
... rev 1

... depotFile //depot/file2
... rev 2
"#;
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.records[0].get("depotFile").unwrap(), "//depot/file1");
        assert_eq!(parsed.records[0].get("rev").unwrap(), "1");
        assert_eq!(parsed.records[1].get("depotFile").unwrap(), "//depot/file2");
        assert_eq!(parsed.records[1].get("rev").unwrap(), "2");
    }
}
