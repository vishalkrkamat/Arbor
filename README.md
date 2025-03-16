# 🌳 Arbor - Terminal File Manager  

Arbor is a simple and interactive terminal-based file manager built using Rust, powered by **ratatui** for the UI and **crossterm** for handling user input. It allows users to navigate directories, view file contents, and explore folder structures.

## 📜 Features  

- 📂 **Navigate Directories** (Use `h` to go back, `l` to go forward)  
- 📄 **View File Contents** (Open files to preview their contents in the terminal)  
- 🔼🔽 **Move Up & Down** (Use `j` and `k` to move through files and directories)  
- 🚀 **Smooth Terminal UI** (Uses **ratatui** for a clean TUI experience)  

## 🎮 Controls  

| Key | Action |
|-----|--------|
| `j` | Move down in the list |
| `k` | Move up in the list |
| `h` | Go to the previous directory |
| `l` | Enter the selected directory |
| `q` | Quit the application |
| `d` | Delete Functionality |
| `r` | Rename Functionality |
| `a` | Create Dir/Files |

## 🛠️ Installation  

### Prerequisites  
Ensure you have **Rust** installed. If not, install it using:  
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone & Build  
```sh
git clone https://github.com/yourusername/Arbor.git
cd Arbor
cargo build --release
```

### Run  
```sh
cargo run
```
## Documentation incomplete as project is in very early stage

## 📜 License  

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

