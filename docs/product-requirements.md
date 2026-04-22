# Product Requirements: Bitcoin Knots/Core Web Console

__Product Name:__ BitNode Console
__Version:__ 1.0
__Date:__ 21st April 2026
__Author:__ Ian Teda <ian+bitnode@teda.id.au>

## 1. Introduction

### 1.1 Product Vision

To provide Bitcoin Knots/Core users with an intuitive, secure, and feature-rich web-based interface for managing their Bitcoin daemon/server, simplifying configuration changes, configuration profiles, monitoring status, and viewing logs without requiring command-line interaction.

### 1.2 Problem Statement

Currently, managing Bitcoin Knots/Core requires understanding and access to a Command-Line Interface (CLI) for tasks such as changing settings (e.g., bitcoin.conf), checking daemon status, or viewing log files. This can be a barrier for less technical users, making configuration, troubleshooting, and general daemon management more difficult.

Bitcoin Knots/Core has sensibly built-in defaults, designed to minimise understanding and avoid mistakes. Power users may like a more customizable configuration.

Additionally, you may have users who want to remain anonymous, run on a lower-spec machine, or operate a full public node. All require different configuration profiles that can be suggested.

Configuration flags and setting names are not always evident in their functionality, nor are the implications of changing them. The `bitcoin.conf` file can become large and complex to navigate if notes for each setting are included. 

### 1.3 Target Audience

- __Beginner Bitcoin Knots/Core Users:__ Individuals new to running a full node who find the CLI intimidating.
- __Intermediate Users:__ Those who use CLI but would prefer a more streamlined and visual way to manage everyday tasks.
- __Advanced Users:__ Those who want to set up a specific profile or use case for their node.
- __Node Operators:__ Individuals or small organisations running dedicated Bitcoin nodes who desire easier remote management (with appropriate security measures).
- __Developers/Testers:__ For quick inspection and modification of testnet/regtest environments.

### 1.4 Goals & Objectives

- __Increase Accessibility:__ Reduce the technical barrier for managing Bitcoin Core.
- __Enhance Usability:__ Provide a more intuitive and visually appealing interface for common operations.
- __Streamline Configuration:__ Offer an easy way to view and modify `bitcoin.conf` settings.
- __Improve Monitoring:__ Provide real-time insights into daemon status and resource usage.
- __Simplify Troubleshooting:__ Centralise log viewing and filtering.
- __Maintain Security:__ Ensure the web console adheres to best security practices for exposing daemon functionality.


## 2. Phases

It is anticipated that there will be three stages to development, which will have their own git branch within the repository:

1. __Feasibility Stage:__ This stage will demonstrate the underlying technologies and their suitability for achieving the desired outcomes.
2. __Concept Stage:__ This stage will be used to develop a minimal viable product that demonstrates functionality and use cases.
3. __Development Stage:__ This stage will be used to fill out the functionality and user experience. While refining the code and security aspects.

## 3. Scope

### 3.1 Feasibility Stage Features:

Dashboard (Bitcoin Core RPC Integration):
Display the status of the following services: VPN, Tor, I2P, and Bitcoin.Knots/Core daemon.
List the number of peers.
Graph the status of the blockchain verification.
Log Viewer (Systemd-Journalctl Integration):
Display systemd-journalctl output
Daemon Settings:
List out settings with toggles, lists, etc. (minimal/common).
Read the `bitcoin.conf` file.
Write to the `bitcoin.conf` file.
3.1 Concept Stage - In-Scope Features
Daemon Status Dashboard:
Current Block Height (synced vs. latest known).
Network Connections (peers, active connections).
Network Connectivity (vpn, Tor, I2Pd, CJDNS status).
Disk Usage (blockchain size, prune status).
Synchronisation Progress (percentage).
Uptime.
Version Information.
Status indicators (running, stopped, syncing, error).
Settings Management:
Settings are organised into relevant sections, with a title, description, default value, implications, and reasons for potential changes.
Modify key bitcoin.conf parameters (e.g., prune, maxconnections, rpcuser, rpcpassword, server, debuglogfile).
Save changes to bitcoin.conf.
Restart the daemon after saving changes (with confirmation).
Data Directory Path
Configuration File Path
Log Viewer:
Display the contents of debug.log in a scrollable, searchable interface.
Filter logs by level (INFO, WARN, ERROR, DEBUG, TRACE).
Option to "tail -f" (real-time updates).
Daemon Control:
Start, stop, or restart the Bitcoin Core daemon (requires appropriate system permissions).
Graceful shutdown option.
User Authentication & Authorization:
Secure login with username and password (configurable).
HTTPS only.
Session management.
3.1 Development Stage - In-Scope Features
Daemon Status Dashboard:
Current Block Height (synced vs. latest known).
Network Connections (peers, active connections).
Network Connectivity (vpn, Tor, I2Pd, Cjdns status).
Memory Pool Size (transactions, total bytes).
Disk Usage (blockchain size, prune status).
Synchronisation Progress (percentage).
Uptime.
Version Information.
Status indicators (running, stopped, syncing, error).
Settings Management:
Settings are organised into relevant sections, with a title, description, default value, implications, and reasons for potential changes.
Modify key bitcoin.conf parameters (e.g., prune, maxconnections, rpcuser, rpcpassword, server, debuglogfile).
Save changes to bitcoin.conf.
Restart the daemon after saving changes (with confirmation).
Input validation for settings (e.g., port numbers, boolean values).
Network (mainnet, testnet, regtest)
Data Directory Path
Configuration File Path
Log Viewer:
Display the contents of debug.log in a scrollable, searchable interface.
Filter logs by level (INFO, WARN, ERROR, DEBUG, TRACE).
Filter logs by keyword/string.
Option to "tail -f" (real-time updates).
Download the whole log file.
Daemon Control:
Start, stop, or restart the Bitcoin Core daemon (requires appropriate system permissions).
Graceful shutdown option.
User Authentication & Authorization:
Secure login with username and password (configurable).
HTTPS only.
Session management.
3.3 Development Stage - Out-of-Scope Features (for V1.0)
Wallet management (including sending and receiving, and address management).
Mempool visualisation/detailed transaction view.
Advanced RPC command execution interface (beyond exposed settings).
Multi-node management from a single console.
Detailed P2P network topology visualisation.
Backup and restore of `bitcoin.conf` or the data directory.
Automatic software updates for Bitcoin Core.

4. User Stories / Use Cases
As a beginner user, I want to:
See at a glance whether my node is running and synced, so I know its status.
Easily change my `bitcoin.conf` settings, such as maxconnections, without editing a text file, so I can optimise my node.
Restart my node from the web interface after changing settings, so I don't have to use the command line.
View the journalctl log to understand what my node is doing, especially if I suspect an issue.
As an intermediate user, I want to:
Securely access my node's console from a web browser to manage it remotely.
Quickly filter log messages to diagnose specific issues, saving time.
Confirm that my node's current network settings are correct.
As a node operator, I want to:
Monitor key performance indicators, such as block height, peer count, and mempool size, from a dashboard.
Be able to gracefully shut down or restart my node for maintenance.
Have an authenticated and encrypted connection to my console.
4. Functional Requirements
FR.1: Dashboard Display: The system shall display the current block height, the number of connected peers, the mempool size (transactions and bytes), the synchronisation progress, the node uptime, and the Bitcoin Core version on the main dashboard.
FR.2: Daemon Status Indicators: The system shall visually indicate the current state of the Bitcoin Core daemon (running, stopped, syncing, error).
FR.3: `bitcoin.conf` Read: The system shall read and display the current values of configurable bitcoin.conf parameters.
FR.4: `bitcoin.conf` Write: The system shall allow modification of specific bitcoin.conf parameters and persist changes to the `bitcoin.conf` file.
FR.5: Setting Validation: The system shall validate user input for `bitcoin.conf` settings (e.g., integer ranges, boolean values, valid string formats).
FR.6: Daemon Start/Stop/Restart: The system shall provide controls to start, stop, and restart the Bitcoin Core daemon. This functionality will require appropriate OS-level permissions.
FR.7: Log File Display: The system shall display the contents of the debug.log file in a readable format.
FR.8: Log Filtering: The system shall allow filtering of log entries by severity (INFO, WARN, ERROR, DEBUG, TRACE) and by arbitrary text string.
FR.9: Log Tail: The system shall provide an option to view log updates in real-time (similar to tail -f).
FR.10: Log Download: The system shall provide a mechanism to download the complete debug.log file.
FR.11: User Authentication: The system shall require users to authenticate with a username and password to access the console.
FR.12: Secure Connection: The web console shall enforce HTTPS for all communication.
FR.13: Session Management: The system shall implement secure session management (e.g., token-based, timeout).
FR.14: Data Directory Display: The system shall display the configured Bitcoin Core data directory path.
FR.15: Config File Path Display: The system shall display the path to the `bitcoin.conf` file being managed.
FR.16: Prometheus Metrics: Expose a Prometheus metrics endpoint for interfacing with Prometheus
5. Non-Functional Requirements
NFR.1: Security:
Authentication: Must support strong, configurable passwords.
Authorisation: The web console should run with the minimum necessary privileges to interact with Bitcoin Core.
Encryption: All web traffic must use HTTPS/TLS 1.2+ for encrypted communication.
Input Sanitisation: All user inputs must be rigorously sanitised to prevent injection attacks (e.g., XSS, command injection).
Rate Limiting: Implement rate limiting on login attempts to prevent brute-force attacks.
Access Control: By default, the web console should be accessible only from localhost or explicitly configured IP addresses.
NFR.2: Performance:
Dashboard data should refresh within 5 seconds.
Log viewing and filtering should be responsive, even with large log files (e.g., rendering the last 1000 lines instantly and allowing scrolling to older entries).
Setting changes and daemon control commands should execute within 2 seconds.
NFR.3: Usability:
Intuitive UI: The interface must be clean, easy to navigate, and consistent in its design.
Clear Feedback: Provide clear visual feedback for actions (e.g., "Settings saved," "Node restarting").
Error Handling: Display user-friendly error messages when operations fail.
Responsiveness: The UI should be responsive across standard desktop browser sizes.
NFR.4: Reliability:
The web console should remain operational even if the Bitcoin Core daemon stops or crashes.
Changes to `bitcoin.conf` must be atomic to prevent file corruption.
NFR.5: Maintainability:
The codebase should be modular and well-documented.
Dependencies should be clearly managed and regularly updated.
NFR.6: Compatibility:
Browsers: Compatible with modern web browsers (Chrome, Firefox, Safari, Edge).
Operating Systems: Compatible with standard Linux distributions where Bitcoin Core is typically run (Ubuntu, Debian, Fedora, Arch).
Bitcoin Core Versions: Aim for compatibility with the latest stable Bitcoin Core releases.
6. System Architecture (High-Level)
Frontend: Single Page Application (SPA) using a modern JavaScript framework (e.g., React, Vue, Angular) or plain HTML/CSS/JS.
Backend: A lightweight Rust Tonic server that acts as an intermediary between the frontend and the Bitcoin Knots/Core daemon.
Communication:
Frontend to Backend: gRPC API over HTTPS.
Backend to Bitcoin Core: Primarily using Bitcoin Knots/Core's RPC interface. For `bitcoin.conf`, direct file system access will be required (with careful permission management). For log reading, access to systemd-journal through [rust-systemd](https://github.com/codyps/rust-systemd)
Persistence: Configuration settings for the web console (e.g., user credentials, port) will be stored in a separate, secure configuration file specific to the web console application itself.
7. Dependencies
Bitcoin Knots/Core Daemon: Must be installed and running on the same machine or an accessible machine.
Operating System: Linux (primary target).
RPC Interface: The web console relies heavily on Bitcoin Knot's/Core's RPC interface.
File System Access: Requires read/write access to bitcoin.conf and read access to debug.log.
Systemd Access: Requires access to systemd dbus for checking status and restarting services.
Process Management: Requires the ability to send SIGTERM (for graceful shutdown) or SIGKILL (for forced stop) to the Bitcoin Knots/Core process. [rust-systemd](https://github.com/codyps/rust-systemd)
8. Assumptions and Constraints
8.1 Assumptions
Users have basic knowledge of Bitcoin Core's purpose.
The web console will be deployed on a system with adequate resources (CPU, RAM, disk space) to run both Bitcoin Knots/Core and the console.
Users understand the security implications of exposing a web interface to their node, especially for remote access.
The web console will be installed on the same machine as the Bitcoin Core daemon, or on a trusted network segment with strict firewall rules.
8.2 Constraints
Security First: All design and implementation decisions must prioritise security.
Minimal Footprint: The web console should be lightweight and not significantly impact the performance of the Bitcoin Knots/Core daemon.
No Wallet Features (V1): Explicitly excluded to simplify V1 and reduce security surface.
OS Permissions: The web console service requires appropriate OS permissions to interact with the `bitcoin.conf` file, systemd and the Bitcoin Knots/Core process. This will require careful setup.
9. Future Considerations / Roadmap (Beyond V1.0)
Wallet Integration (Read-Only/Basic): Display wallet balance, transaction history (no send/receive).
Advanced RPC Console: A simple interface to execute arbitrary RPC commands.
Multi-Node Management: Ability to manage multiple Bitcoin Core instances from one console.
Notification System: Alerts for significant events (e.g., node offline, new block, low disk space).
Theming/Customisation: Dark mode, custom themes.
Docker Containerization: Provide official Docker images for easy deployment.
Performance Metrics Graphs: Visualise historical data on block height, peer count, mempool size, and more.
Peer Information: Detailed view of connected peers (IP, version, services, etc.).