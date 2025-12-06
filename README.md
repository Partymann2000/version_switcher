# **🔄 Version Switcher**

A modern, lightweight Windows desktop application built with **Rust** to easily manage and switch between different versions of programming languages (like Python, Node.js, PHP, Java) or any other command-line tools.

*(Add a screenshot of your app here)*

## **🚀 Why this tool?**

Developing with multiple versions of the same language can be a pain on Windows. Changing environment variables manually is tedious and error-prone.

**Version Switcher** solves this by:

1. **Grouping** versions by language (e.g., "Python", "NodeJS").
2. **Validating** paths automatically (checks if the folder actually exists).
3. **Hot-Swapping** the User `PATH` variable instantly without needing a system restart.
4. **Notifying** the system and the user (via Windows Toast Notification) upon successful switch.

## **✨ Features**

* **GUI based on `egui`:** Fast, responsive, and lightweight.
* **Path Validation:** Instantly see if a configured path is valid (✅) or missing (❌).
* **Folder Picker:** Easily select directories using the native Windows file dialog.
* **Instant Activation:** Updates the `HKEY_CURRENT_USER\Environment\Path` registry key and broadcasts the change to running applications.
* **Visual Feedback:** Green indicators show exactly which version is currently active in your system PATH.
* **Persistence:** Remembers your configuration between restarts.
* **Native Notifications:** Get a desktop popup when the version switch is complete.

## **🛠️ Installation & Build**

### **Prerequisites**

* [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
* Windows OS (tested on Windows 10/11)

### **Building from Source**

1. **Clone the repository:**  
   ```
    git clone \[https://github.com/your-username/version-switcher.git\](https://github.com/your-username/version-switcher.git)  
    cd version-switcher
   ```

2. **Build the Release version:**  
   ```
    cargo build --release
   ```

   *Note: This will also compile the application icon into the executable.*
3. Run:  
   The executable will be located at:  
   `target/release/version_switcher.exe`

## **📖 How to Use**

1. **Create a Group:**
    * Enter a name (e.g., `Python`) in the "New Group" field and click the button.
2. **Add a Version:**
    * Select your group from the dropdown.
    * **Name:** Give it a friendly alias (e.g., `3.11.0`).
    * **Path:** Paste the path to the binary folder or use the **📂 Folder Button** to browse.
    * Click **"➕ Add"**.
3. **Switch:**
    * Click the **"Activate"** button next to the version you want to use.
    * A notification will appear, and the status indicator will turn green (🟢).
    * Open a *new* terminal window to use the switched version.

## **💻 Tech Stack**

* **Language:** [Rust](https://www.rust-lang.org/)
* **GUI:** [eframe / egui](https://github.com/emilk/egui)
* **Registry Access:** `winreg`
* **System Calls:** `winapi`
* **Notifications:** `notify-rust`
* **Serialization:** `serde`

## **⚠️ Note on Environment Variables**

This tool modifies the **User** Path variable (`HKCU\Environment\Path`). It does **not** touch the System Path (which requires Admin privileges). This is generally safer and sufficient for development environments.

## **📄 License**

This project is licensed under the MIT License \- see the [LICENSE](https://www.google.com/search?q=LICENSE) file for details.