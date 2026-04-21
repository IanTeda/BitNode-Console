# BitNode Console

BitNode Console is a web-based interface for managing a Bitcoin Knots/Core daemons. It aims to simplify configuration changes, provide configuration profiles, monitor daemon status, and view logs without needing to enter the command line.

## 1. Overview

Managing the Bitcoin Knots/Core daemon typically requires understanding and access to the host computer or server's Command-Line Interface (CLI) to perform tasks such as changing settings (i.e. editing the `bitcoin.conf`), checking the daemon's status (i.e. checking the status in systemctl), or viewing log files (i.e. viewing journcalctl). This can be a barrier for less technical users, making and tracking configuration changes, troubleshooting, and general daemon management. Becoming a hurdle for running a Bitcoin node.

While Bitcoin Knots/Core provides thoughtful default settings that simplify usage and help prevent mistakes, experienced users may want more control over their configuration. BitNode aims to offer an extensive list of configurable options, provide useful advice and warnings as changes are made, track changes, and offer configuration profiles as a starting point for users who want more control over their configuration. Furthermore, the purpose and effects of many of these configuration flags and setting names can be unclear, making guidance especially valuable.

Additionally, the `bitcoin.conf` file can grow large and difficult to manage. BitNode offers a logical, structured way to configure a Bitcoin Knots/Core daemon, including dedicated areas for notes and references to aid navigation.

## 2. Features

BitNode Console features will be added over time, as discussed in the roadmap section below. For now, the aim of the feasibility stage is to have the following features available:

- Web-based console
- Authentication based on the RPC username and password.
- Status dashboard.
- Log view, including useful filters and alerts.

## 3. Getting Started

No information available yet. Details will be added during the concept stage.

## 4. Usage

No information available yet. Details will be added during the concept stage.

## 5. Configuration

No information available yet. Details will be added during the concept stage.

## 6. Project Structure

No information available yet. Details will be added during the feasibility stage.

## 7. Development

This section describes the technology stack used and how to set up your development environment if you want to contribute or play around.

### Technology Stack

The BitNode uses the following technology stack:

- Rust Language: Because it is what I am learning.

### Set Up for Development

No information available yet. Details will be added during the concept stage.

TODO: Add git flow repository structure

### Testing

No information available yet. Details will be added during the feasibility stage.

## 8. Roadmap

It is anticipated that there will be three stages to the initial development of BitNode, which will have their own git release branch within the repository:

1. __Feasibility Stage:__ This stage will demonstrate the underlying technologies and their suitability for achieving the desired outcomes.
2. __Concept Stage:__ This stage will be used to develop a minimal viable product that demonstrates functionality and use cases.
3. __Development Stage:__ This stage will be used to fill out the functionality and user experience. While refining the code and security aspects.

### 8.1. Feasibility Stage

The feasibility stage will initially aim to demonstrate the following:

1. Log in to the console using the RPC username and password (i.e. authenticating via the daemon)
2. Displays the status of the Bitcoin.Knots/Core daemon.
3. Display systemd-journalctl output
4. Provide a logical list of current node settings (read and parse the bitcoin.con file)

### 8.2 Concept Stage

No information available yet. Details will be added during the concept stage.

### 8.3 Development Stage

No information available yet. Details will be added during the development stage.

## License

License type [GENERAL PUBLIC LICENSE Version 3, 29 June 2007](LICENSE)