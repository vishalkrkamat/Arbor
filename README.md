# 🌳 Arbor - Terminal File Manager

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
Arbor is an interactive, three-pane terminal-based file manager built using Rust. It leverages **ratatui** for the Terminal User Interface (TUI) and **crossterm** for event handling. It allows users to navigate directories, preview files, manage files and folders (create, rename, delete), and perform copy/paste operations.

## 📜 Features

* **📊 Three-Pane Layout:** Displays Parent Directory, Current Directory, and a Preview pane (showing subdirectory contents or file previews).
* **📂 Navigate Directories:** Seamlessly move between directories.
* **📄 File Preview:** View contents of text files directly in the terminal. Detects binary files.(Lots of file format has to be added for better preview underdeveopment)
* **🔼🔽 Smooth Navigation:** Use familiar keybindings (`j`, `k`, `h`, `l`) to move through items and hierarchies.
* **📝 File Management:**
    * **Delete:** Remove files or directories (with confirmation).
    * **Rename:** Rename files or directories via an interactive prompt.
    * **Create:** Create new files or directories (supports nested creation like `mkdir -p`) via an interactive prompt.
* **✨ Selection Mode:** Enter a visual selection mode (`v`) to select multiple items for batch operations (like mass deletion).
* **📋 Copy & Paste:** Copy single files/directories and paste them into the current directory (recursive directory copy).
* **🚀 Interactive Popups:** Handles confirmations, renaming, and creation through clean TUI popups.
* 
## 🎮 Controls

Arbor operates primarily in **Normal Mode**. Some actions involve temporary popups or switching to **Selection Mode**.

### Normal Mode

| Key         | Action                                                                |
| :---------- | :-------------------------------------------------------------------- |
| `j` / `↓`   | Move focus down in the current directory list                         |
| `k` / `↑`   | Move focus up in the current directory list                           |
| `h` / `←`   | Go to the parent directory                                            |
| `l` / `→`   | Enter the selected directory / Preview selected file                  |
| `d`         | Initiate delete for the selected item (opens confirmation popup)      |
| `r`         | Initiate rename for the selected item (opens rename prompt)           |
| `a`         | Initiate create file/directory (opens creation prompt)                |
| `y`         | Copy (Yank) the selected file or directory                            |
| `p`         | Paste the copied item(s) into the current directory                   |
| `v`         | Enter **Selection Mode** (visually select multiple items)             |
| `q`         | Quit the application                                                  |

### Selection Mode (Enter with `v`, Exit with `Esc`)

| Key         | Action                                                                |
| :---------- | :-------------------------------------------------------------------- |
| `j` / `↓`   | Move down and toggle selection status of the item                     |
| `k` / `↑`   | Move up and toggle selection status of the item                       |
| `d`         | Initiate delete for *all* selected items (opens confirmation popup)   |
| `Esc`       | Exit Selection Mode and return to Normal Mode (clears selection)      |
| `q`         | Quit the application                                                  |
### Popup Controls (Confirmation / Rename / Create Prompts)

| Key         | Action                                                                |
| :---------- | :-------------------------------------------------------------------- |
| `Enter`     | Confirm action / Submit input                                         |
| `Esc`       | Cancel the action and close the popup                                 |
| `y`         | Confirm action (in Delete confirmation popup)                         |
| `n`         | Deny action (in Delete confirmation popup)                            |
| `Backspace` | Delete the last character in the input field (Rename/Create)          |
| `[Any Char]`| Type character into the input field (Rename/Create)                   |

## ⚠️ Development Status & Running the Application

This project is currently under **rapid development**. Expect frequent changes, potentially breaking ones, and know that major improvements are still needed to enhance stability and feature completeness. 

## 🛠️ Installation

### Prerequisites

Ensure you have **Rust** and **Cargo** installed. If not, install them using [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
# Follow the on-screen instructions
```

### Clone & Build  
```sh
git clone https://github.com/vishalkrkamat/Arbor.git
cd Arbor
cargo build --release
```

### Run
```
cargo run
```
