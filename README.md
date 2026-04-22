<!-- markdownlint-disable MD033 -->
<!-- Improved compatibility of back to top link -->
<a name="readme-top"></a>

<!-- The extra return after center is needed for it to render the markdown links center -->
<div align="center">

[![License][license-shield]][license-url]
[![Issues][issues-shield]][issues-url]
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]

</div>

<br />

<!-- PROJECT HEADER -->
<div align="center">
    <a href="https://github.com/IanTeda/BitNode-Console">
        <img src="https://github.com/IanTeda/BitNode-Console/raw/develop/docs/images/logo.png" alt="Logo" width="80" height="80">
    </a>
    <h1 align="center">BitNode Console</h1>
    <p align="center">
        BitNode Console is a web-based interface for managing a Bitcoin Knots/Core daemon node. It aims to simplify configuration changes, provide configuration profiles, monitor daemon status, and view logs without needing to enter the command line.
    <br />
    <a href="https://ianteda.github.io/BitNode-Console/">Explore the Docs</a>
    ·
    <a href="https://ianteda.github.io/BitNode-Console/issues">Report a Bug</a>
    ·
    <a href="https://ianteda.github.io/BitNode-Console/issues">Request a Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#overview">Project Overview</a></li>
    <li><a href="#features">Application Features</a></li>
    <li><a href="#quick-start">Application Quick Start</a></li>
    <li><a href="#configuration">Application Configuration</a></li>
    <li><a href="#repository-structure">Repository Structure</a></li>
    <li>
      <a href="#development">Code Development</a>
      <ul>
        <li><a href="#technology-stack">Application Technology Stack</a></li>
        <li><a href="#repository-structure">Repository Structure</a></li>
        <li><a href="#getting-started">Code Getting Started</a></li>
        <li><a href="#testing">Code Testing</a></li>
      </ul>
    </li>
    <li>
        <a href="#roadmap">Project Roadmap</a>
        <ul>
            <li><a href="#feasibility-stage">Feasibility</a></li>
            <li><a href="#concept-stage">Concept Stage</a></li>
            <li><a href="#development-stage">Development Stage</a></li>
        </ul>
    </li>
    <li><a href="#license">License</a></li>
  </ol>
</details>


<!-- REPOSITORY OVERVIEW -->
## Overview

[![BitNode Screenshot][product-screenshot]](https://github.com/IanTeda/BitNode-Console)

Managing the Bitcoin Knots/Core daemon typically requires understanding and access to the host computer or server's Command-Line Interface (CLI) to perform tasks such as changing settings (i.e. editing the `bitcoin.conf`), checking the daemon's status (i.e. checking the status in systemctl), or viewing log files (i.e. viewing journcalctl). This can be a barrier for less technical users, making and tracking configuration changes, troubleshooting, and general daemon management. Becoming a hurdle for running a Bitcoin node.

While Bitcoin Knots/Core provides thoughtful default settings that simplify usage and help prevent mistakes, experienced users may want more control over their configuration. BitNode aims to offer an extensive list of configurable options, provide useful advice and warnings as changes are made, track changes, and offer configuration profiles as a starting point for users who want more control over their configuration. Furthermore, the purpose and effects of many of these configuration flags and setting names can be unclear, making guidance especially valuable.

Additionally, the `bitcoin.conf` file can grow large and difficult to manage. BitNode offers a logical, structured way to configure a Bitcoin Knots/Core daemon, including dedicated areas for notes and references to aid navigation.

<!-- APPLICATION FEATURES -->
## Features

BitNode Console features will be added over time, as discussed in the roadmap section below. For now, the aim of the feasibility stage is to have the following features available:

- Web-based console
- Authentication based on the RPC username and password.
- Status dashboard.
- Log view, including useful filters and alerts.

<!-- APPLICATION QUICK START -->
## Quick Start

No information available yet. Details will be added during the concept stage.

<!-- APPLICATION CONFIGURATION -->
## Configuration

No information available yet. Details will be added during the concept stage.

<!-- APPLICATION DEVELOPMENT -->
## Development

This section describes the technology stack used and how to set up your development environment if you want to contribute or play around.

<!-- APPLICATION TECHNOLOGY STACK -->
### Technology Stack

The BitNode uses the following technology stack:

- Rust Language: Because it is what I am learning.

<!-- REPOSITORY FILE/FOLDER STRUCTURE -->
### Repository Structure

No information available yet. Details will be added during the feasibility stage.

```text


```

<!-- GETTING STARTED WITH DEVELOPMENT/CONTRIBUTING -->
### Getting Started

No information available yet. Details will be added during the concept stage.

TODO: Add git flow repository structure

<!-- CODE TESTING NOTES -->
### Testing

No information available yet. Details will be added during the feasibility stage.

<!-- APPLICATION ROADMAP -->
## Roadmap

It is anticipated that there will be three stages to the initial development of BitNode, which will have their own git release branch within the repository:

1. __Feasibility Stage:__ This stage will demonstrate the underlying technologies and their suitability for achieving the desired outcomes.
2. __Concept Stage:__ This stage will be used to develop a minimal viable product that demonstrates functionality and use cases.
3. __Development Stage:__ This stage will be used to fill out the functionality and user experience. While refining the code and security aspects.

<!-- FEASIBILITY STAGE -->
### Feasibility Stage

The feasibility stage will initially aim to demonstrate the following:

1. Log in to the console using the RPC username and password (i.e. authenticating via the daemon)
2. Displays the status of the Bitcoin.Knots/Core daemon.
3. Display systemd-journalctl output
4. Provide a logical list of current node settings (read and parse the bitcoin.con file)

<!-- CONCEPT STAGE -->
### Concept Stage

No information available yet. Details will be added during the concept stage.

<!-- DEVELOPMENT STAGE -->
### Development Stage

No information available yet. Details will be added during the development stage.

<!-- CODE LICENSE -->
## License

License type [GPL V3](LICENSE)

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/IanTeda/BitNode-Console.svg?style=for-the-badge
[contributors-url]: https://github.com/IanTeda/BitNode-Console/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/IanTeda/BitNode-Console.svg?style=for-the-badge
[forks-url]: https://github.com/IanTeda/BitNode-Console/network/members
[stars-shield]: https://img.shields.io/github/stars/IanTeda/BitNode-Console.svg?style=for-the-badge
[stars-url]: https://github.com/IanTeda/BitNode-Console/stargazers
[issues-shield]: https://img.shields.io/github/issues/IanTeda/BitNode-Console.svg?style=for-the-badge
[issues-url]: https://github.com/IanTeda/BitNode-Console/issues
[license-shield]: https://img.shields.io/github/license/IanTeda/BitNode-Console.svg?style=for-the-badge
[license-url]: https://github.com/IanTeda/BitNode-Console/blob/master/LICENSE.txt
[product-screenshot]: https://github.com/IanTeda/BitNode-Console/raw/develop/docs/images/screenshot.png