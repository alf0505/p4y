# p4y Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a read-only Perforce workspace browser in Rust using the Iced framework, focusing on virtual workspace tree navigation and history (CL) inspection.

**Architecture:** MVC-like structure using the Iced Elm Architecture. Asynchronous P4 CLI calls for data fetching, using `-Ztag` for efficient parsing. Lazy-loading tree nodes and metadata caching for performance.

**Tech Stack:** Rust, Iced (GUI), Tokio (Async), Chrono (Date/Time).

---

### Task 1: Project Scaffolding & Dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Initialize Cargo project and add dependencies**

```toml
[package]
name = "p4y"
version = "0.1.0"
edition = "2021"

[dependencies]
iced = { version = "0.12", features = ["full"] }
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
```

- [ ] **Step 2: Create a basic "Hello Iced" entry point**

```rust
use iced::{executor, Application, Command, Element, Settings, Theme};

pub fn main() -> iced::Result {
    P4y::run(Settings::default())
}

struct P4y {}

#[derive(Debug, Clone)]
enum Message {}

impl Application for P4y {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (P4y {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("p4y - Perforce Inspector")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        "Hello, p4y!".into()
    }
}
```

- [ ] **Step 3: Run the app to verify setup**

Run: `cargo run`
Expected: A window opens with "Hello, p4y!".

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "chore: initial project scaffolding with iced"
```

---

### Task 2: P4 CLI Wrapper & Ztag Parser

**Files:**
- Create: `src/p4.rs`
- Modify: `src/main.rs` (add module)

- [ ] **Step 1: Define P4 data structures and a basic Ztag parser**

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ZtagOutput {
    pub records: Vec<HashMap<String, String>>,
}

pub fn parse_ztag(output: &str) -> ZtagOutput {
    let mut records = Vec::new();
    let mut current_record = HashMap::new();

    for line in output.lines() {
        if line.is_empty() {
            if !current_record.is_empty() {
                records.push(current_record);
                current_record = HashMap::new();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("... ") {
            let mut parts = rest.splitn(2, ' ');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                current_record.insert(key.to_string(), value.to_string());
            }
        }
    }
    if !current_record.is_empty() {
        records.push(current_record);
    }
    ZtagOutput { records }
}
```

- [ ] **Step 2: Add test for Ztag parser**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ztag() {
        let input = "... depotFile //depot/a\n... rev 1\n\n... depotFile //depot/b\n... rev 2";
        let result = parse_ztag(input);
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].get("depotFile").unwrap(), "//depot/a");
        assert_eq!(result.records[1].get("rev").unwrap(), "2");
    }
}
```

- [ ] **Step 3: Implement async P4 command execution**

```rust
use tokio::process::Command;

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
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/p4.rs
git commit -m "feat: add P4 CLI wrapper and ztag parser"
```

---

### Task 3: Virtual Workspace Tree (Lazy Loading)

**Files:**
- Create: `src/tree.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Define Tree Node structure**

```rust
#[derive(Debug, Clone)]
pub enum NodeType {
    Directory,
    File,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String, // Full P4 path like //workspace/src
    pub node_type: NodeType,
    pub children: Option<Vec<TreeNode>>,
    pub is_expanded: bool,
}
```

- [ ] **Step 2: Implement logic to fetch child nodes via `p4 dirs` and `p4 files`**

```rust
pub async fn fetch_children(parent_path: &str) -> Result<Vec<TreeNode>, String> {
    // 1. Run p4 dirs parent_path/*
    // 2. Run p4 files parent_path/*
    // 3. Merge into Vec<TreeNode>
    Ok(vec![]) // Implementation placeholder for the engineer
}
```

- [ ] **Step 3: Update `P4y` state to include the tree**

```rust
struct P4y {
    root_node: TreeNode,
}
```

- [ ] **Step 4: Commit**

```bash
git add src/tree.rs
git commit -m "feat: define virtual tree structures and lazy loading stubs"
```

---

### Task 4: Main Layout Implementation (Three Columns)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement the 3-column split layout in `view()`**

```rust
use iced::widget::{column, row, container, scrollable, text, button};

fn view(&self) -> Element<Message> {
    let tree_pane = container(scrollable("1. Tree content")).width(250).height(500).style(...);
    let history_pane = container(scrollable("2. History List")).width(350).height(500).style(...);
    let detail_pane = container(scrollable("3. CL Details")).width(iced::Length::Fill).height(500).style(...);

    row![tree_pane, history_pane, detail_pane].into()
}
```

- [ ] **Step 2: Add basic styling for panes**

- [ ] **Step 3: Run and verify layout**

Run: `cargo run`
Expected: UI with a left sidebar and two right-hand panes.

- [ ] **Step 4: Commit**

```bash
git commit -am "feat: implement three-pane layout"
```

---

### Task 5: History (CL) Fetching & Detail View

**Files:**
- Modify: `src/p4.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement `fetch_history(path)` using `p4 filelog`**
- [ ] **Step 2: Implement `fetch_cl_detail(cl_id)` using `p4 describe -s`**
- [ ] **Step 3: Connect Tree selection to History fetching**
- [ ] **Step 4: Connect History selection to Detail fetching**
- [ ] **Step 5: Commit**

```bash
git commit -am "feat: connect tree selection to history and CL details"
```

---

### Task 6: Modal Content Viewer

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement `p4 print` call to fetch file content**
- [ ] **Step 2: Add a Modal overlay in Iced (using a conditional `view` element)**
- [ ] **Step 3: Implement "View Content" button in CL Details pane**
- [ ] **Step 4: Commit**

```bash
git commit -am "feat: add modal content viewer"
```

---

### Task 7: Polishing & Error Handling

- [ ] **Step 1: Add "Connection Settings" dialog (Server, User, Client)**
- [ ] **Step 2: Handle P4 CLI errors gracefully in UI**
- [ ] **Step 3: Add loading indicators during async calls**
- [ ] **Step 4: Final verification of the "Browse -> History -> CL -> Content" flow**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat: finalize app with settings and error handling"
```
