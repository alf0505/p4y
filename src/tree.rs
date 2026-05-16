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

    pub fn is_directory(&self) -> bool {
        self.node_type == NodeType::Directory
    }

    pub fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn update_node(&mut self, target_path: &str, new_children: Vec<TreeNode>) -> bool {
        if self.path == target_path {
            self.children = Some(new_children);
            self.is_expanded = true;
            return true;
        }
        if let Some(ref mut children) = self.children {
            for child in children {
                if child.update_node(target_path, new_children.clone()) {
                    return true;
                }
            }
        }
        false
    }
}

pub async fn fetch_children(parent_path: &str) -> Result<Vec<TreeNode>, P4Error> {
    let wildcard_path = format!("{}/*", parent_path);

    let (dirs_output, files_output) = tokio::join!(
        run_p4(vec!["dirs", &wildcard_path]),
        run_p4(vec!["files", &wildcard_path])
    );

    let mut children = Vec::new();

    let process_output = |output: Result<String, P4Error>, key: &str, node_type: NodeType| {
        match output {
            Ok(out) => {
                let ztag = parse_ztag(&out);
                let mut nodes = Vec::new();
                for record in ztag.records {
                    if let Some(path) = record.get(key) {
                        let name = path.split('/').last().unwrap_or("").to_string();
                        nodes.push(TreeNode {
                            name,
                            path: path.clone(),
                            node_type: node_type.clone(),
                            children: None,
                            is_expanded: false,
                        });
                    }
                }
                Ok(nodes)
            }
            Err(P4Error::Process(e)) if e.contains("no such file(s)") => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    };

    children.extend(process_output(dirs_output, "dir", NodeType::Directory)?);
    children.extend(process_output(files_output, "depotFile", NodeType::File)?);

    // Sort children by name
    children.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(children)
}

pub fn find_node_mut<'a>(node: &'a mut TreeNode, path: &str) -> Option<&'a mut TreeNode> {
    if node.path == path {
        return Some(node);
    }
    if let Some(ref mut children) = node.children {
        for child in children {
            if let Some(found) = find_node_mut(child, path) {
                return Some(found);
            }
        }
    }
    None
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

    #[test]
    fn test_combine_dirs_and_files_logic() {
        let dirs_output = "... dir //depot/p4y/src\n";
        let files_output = "... depotFile //depot/p4y/Cargo.toml\n";
        
        let mut children = Vec::new();
        
        // Simulating dirs part of fetch_children
        let ztag_dirs = parse_ztag(dirs_output);
        for record in ztag_dirs.records {
            if let Some(path) = record.get("dir") {
                let name = path.split('/').last().unwrap_or("").to_string();
                children.push(TreeNode::new_directory(name, path.clone()));
            }
        }

        // Simulating files part of fetch_children
        let ztag_files = parse_ztag(files_output);
        for record in ztag_files.records {
            if let Some(path) = record.get("depotFile") {
                let name = path.split('/').last().unwrap_or("").to_string();
                children.push(TreeNode::new_file(name, path.clone()));
            }
        }

        children.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Cargo.toml");
        assert_eq!(children[0].node_type, NodeType::File);
        assert_eq!(children[1].name, "src");
        assert_eq!(children[1].node_type, NodeType::Directory);
    }
}
