use std::collections::HashMap;
use tokio::process::Command;
use std::fmt;

#[derive(Debug)]
pub enum P4Error {
    Io(std::io::Error),
    Process(String),
    Utf8(std::string::FromUtf8Error),
}

impl fmt::Display for P4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            P4Error::Io(e) => write!(f, "IO error: {}", e),
            P4Error::Process(e) => write!(f, "P4 process error: {}", e),
            P4Error::Utf8(e) => write!(f, "UTF-8 conversion error: {}", e),
        }
    }
}

impl std::error::Error for P4Error {}

impl From<std::io::Error> for P4Error {
    fn from(error: std::io::Error) -> Self {
        P4Error::Io(error)
    }
}

impl From<std::string::FromUtf8Error> for P4Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        P4Error::Utf8(error)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZtagOutput {
    pub records: Vec<HashMap<String, String>>,
}

/// Parses the output of a Perforce command run with -Ztag.
/// 
/// Ztag output format:
/// ... field1 value1
/// ... field2 value2
/// (empty line)
/// ... field1 value3
/// ... field2 value4
pub fn parse_ztag(output: &str) -> ZtagOutput {
    let mut records = Vec::new();
    let mut current_record = HashMap::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !current_record.is_empty() {
                records.push(std::mem::take(&mut current_record));
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("...") {
            let rest = rest.trim_start();
            let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
            if let Some(key) = parts.next() {
                if !key.is_empty() {
                    let value = parts.next().unwrap_or("").trim().to_string();
                    current_record.insert(key.to_string(), value);
                }
            }
        }
    }

    if !current_record.is_empty() {
        records.push(current_record);
    }

    ZtagOutput { records }
}

#[derive(Debug, Clone, Default)]
pub struct P4Settings {
    pub port: String,
    pub user: String,
    pub client: String,
}

/// Runs a Perforce command with the given arguments and returns the Ztag-formatted output.
pub async fn run_p4(args: Vec<&str>, settings: Option<&P4Settings>) -> Result<String, P4Error> {
    let mut command = Command::new("p4");
    command.arg("-Ztag");
    
    if let Some(s) = settings {
        if !s.port.is_empty() { command.env("P4PORT", &s.port); }
        if !s.user.is_empty() { command.env("P4USER", &s.user); }
        if !s.client.is_empty() { command.env("P4CLIENT", &s.client); }
    }

    let output = command.args(args).output().await?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(P4Error::Process(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Changelist {
    pub id: u32,
    pub author: String,
    pub date: String,
    pub description: String,
}

pub async fn fetch_history(path: &str, settings: Option<P4Settings>) -> Result<Vec<Changelist>, P4Error> {
    let output = run_p4(vec!["changes", "-m", "100", path], settings.as_ref()).await?;
    let ztag = parse_ztag(&output);
    let mut changes = Vec::new();

    for record in ztag.records {
        if let Some(change_str) = record.get("change") {
            if let Ok(id) = change_str.parse::<u32>() {
                changes.push(Changelist {
                    id,
                    author: record.get("user").cloned().unwrap_or_default(),
                    date: record.get("time").cloned().unwrap_or_default(),
                    description: record.get("desc").cloned().unwrap_or_default(),
                });
            }
        }
    }

    Ok(changes)
}

pub async fn fetch_file_content(path_with_rev: &str, settings: Option<P4Settings>) -> Result<String, P4Error> {
    let mut command = Command::new("p4");
    command.arg("print").arg("-q");

    if let Some(s) = settings {
        if !s.port.is_empty() { command.env("P4PORT", &s.port); }
        if !s.user.is_empty() { command.env("P4USER", &s.user); }
        if !s.client.is_empty() { command.env("P4CLIENT", &s.client); }
    }

    let output = command.arg(path_with_rev).output().await?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(P4Error::Process(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChangelistDetail {
    pub id: u32,
    pub author: String,
    pub date: String,
    pub description: String,
    pub affected_files: Vec<String>,
}

pub async fn fetch_cl_detail(cl_id: u32, settings: Option<P4Settings>) -> Result<ChangelistDetail, P4Error> {
    let cl_id_str = cl_id.to_string();
    let output = run_p4(vec!["describe", "-s", &cl_id_str], settings.as_ref()).await?;
    let ztag = parse_ztag(&output);
    
    if let Some(record) = ztag.records.first() {
        let mut affected_files = Vec::new();
        let mut i = 0;
        while let Some(depot_file) = record.get(&format!("depotFile{}", i)) {
            affected_files.push(depot_file.clone());
            i += 1;
        }

        Ok(ChangelistDetail {
            id: cl_id,
            author: record.get("user").cloned().unwrap_or_default(),
            date: record.get("time").cloned().unwrap_or_default(),
            description: record.get("desc").cloned().unwrap_or_default(),
            affected_files,
        })
    } else {
        Err(P4Error::Process("No changelist detail found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ztag_single_record() {
        let output = "... depotFile //depot/file1\n... rev 1\n";
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].get("depotFile").unwrap(), "//depot/file1");
        assert_eq!(parsed.records[0].get("rev").unwrap(), "1");
    }

    #[test]
    fn test_parse_ztag_multiple_records() {
        let output = r#"... depotFile //depot/file1
... rev 1

... depotFile //depot/file2
... rev 2
"#;
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.records[0].get("depotFile").unwrap(), "//depot/file1");
        assert_eq!(parsed.records[1].get("depotFile").unwrap(), "//depot/file2");
    }

    #[test]
    fn test_parse_ztag_empty_value() {
        let output = "... fieldWithNoValue ";
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].get("fieldWithNoValue").unwrap(), "");
    }

    #[test]
    fn test_parse_ztag_extra_whitespace() {
        let output = "  ...   field   value  \n\n  ... field2 value2  ";
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.records[0].get("field").unwrap(), "value");
        assert_eq!(parsed.records[1].get("field2").unwrap(), "value2");
    }

    #[test]
    fn test_parse_ztag_empty_lines_and_noise() {
        let output = "\n\n... key1 value1\n\n\n... key2 value2\n\n";
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.records[0].get("key1").unwrap(), "value1");
        assert_eq!(parsed.records[1].get("key2").unwrap(), "value2");
    }

    #[test]
    fn test_parse_ztag_no_value() {
        let output = "... key1\n... key2  ";
        let parsed = parse_ztag(output);
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].get("key1").unwrap(), "");
        assert_eq!(parsed.records[0].get("key2").unwrap(), "");
    }
}
