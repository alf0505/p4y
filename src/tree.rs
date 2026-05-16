use crate::p4::{run_p4, parse_ztag, P4Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Directory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String, // Full P4 path like //workspace/src
    pub node_type: NodeType,
    pub children: Option<Vec<TreeNode>>,
    pub is_expanded: bool,
}

impl TreeNode {
    pub fn new_directory(name: String, path: String) -> Self {
        Self {
            name,
            path,
            node_type: NodeType::Directory,
            children: None,
            is_expanded: false,
        }
    }

    pub fn new_file(name: String, path: String) -> Self {
        Self {
            name,
            path,
            node_type: NodeType::File,
            children: None,
            is_expanded: false,
        }
    }
}

pub async fn fetch_children(parent_path: &str) -> Result<Vec<TreeNode>, P4Error> {
    let mut children = Vec::new();

    // Fetch directories
    let dirs_output = run_p4(vec!["dirs", &format!("{}/*", parent_path)]).await;
    match dirs_output {
        Ok(output) => {
            let ztag = parse_ztag(&output);
            for record in ztag.records {
                if let Some(path) = record.get("dir") {
                    let name = path.split('/').last().unwrap_or("").to_string();
                    children.push(TreeNode::new_directory(name, path.clone()));
                }
            }
        }
        Err(P4Error::Process(e)) if e.contains("no such file(s)") => {
            // Ignore "no such file(s)" error from p4 dirs
        }
        Err(e) => return Err(e),
    }

    // Fetch files
    let files_output = run_p4(vec!["files", &format!("{}/*", parent_path)]).await;
    match files_output {
        Ok(output) => {
            let ztag = parse_ztag(&output);
            for record in ztag.records {
                if let Some(path) = record.get("depotFile") {
                    let name = path.split('/').last().unwrap_or("").to_string();
                    children.push(TreeNode::new_file(name, path.clone()));
                }
            }
        }
        Err(P4Error::Process(e)) if e.contains("no such file(s)") => {
            // Ignore "no such file(s)" error from p4 files
        }
        Err(e) => return Err(e),
    }

    // Sort children by name
    children.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p4::parse_ztag;

    #[test]
    fn test_tree_node_creation() {
        let dir = TreeNode::new_directory("src".to_string(), "//depot/src".to_string());
        assert_eq!(dir.node_type, NodeType::Directory);
        assert_eq!(dir.name, "src");
        assert_eq!(dir.path, "//depot/src");

        let file = TreeNode::new_file("main.rs".to_string(), "//depot/src/main.rs".to_string());
        assert_eq!(file.node_type, NodeType::File);
        assert_eq!(file.name, "main.rs");
        assert_eq!(file.path, "//depot/src/main.rs");
    }

    #[test]
    fn test_parsing_dirs_output() {
        let output = "... dir //depot/p4y/src\n\n... dir //depot/p4y/tests\n";
        let ztag = parse_ztag(output);
        let mut children = Vec::new();
        for record in ztag.records {
            if let Some(path) = record.get("dir") {
                let name = path.split('/').last().unwrap_or("").to_string();
                children.push(TreeNode::new_directory(name, path.clone()));
            }
        }
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "src");
        assert_eq!(children[1].name, "tests");
    }

    #[test]
    fn test_parsing_files_output() {
        let output = "... depotFile //depot/p4y/Cargo.toml\n... rev 1\n\n... depotFile //depot/p4y/README.md\n... rev 2\n";
        let ztag = parse_ztag(output);
        let mut children = Vec::new();
        for record in ztag.records {
            if let Some(path) = record.get("depotFile") {
                let name = path.split('/').last().unwrap_or("").to_string();
                children.push(TreeNode::new_file(name, path.clone()));
            }
        }
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Cargo.toml");
        assert_eq!(children[1].name, "README.md");
    }
}
