# Docker-Style FS and Process Isolation

A lightweight implementation of Docker-style filesystem and process isolation, along with handling for registry images. It shows core Docker functionalities for educational purposes or lightweight use cases

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
  - [Supported Commands](#supported-commands)
  - [Using `rs-docker.sh` (Windows/MacOS)](#using-rs-dockersh-windowsmacos)
  - [Running the Code on System (Linux)](#running-the-code-on-system-linux)

## Introduction

This project mimics essential Docker functionalities, including filesystem isolation, process isolation, and registry image management. It's designed to help users understand how Docker works under the hood and to provide a lightweight alternative for specific use cases

## Features

- **Filesystem Isolation**: Implements isolated filesystems using `chroot` for containers
- **Process Isolation**: Ensures processes run in separate namespaces using `unshare`
- **Registry Image Handling**: Supports pulling and storing images from a registry

## Prerequisites

Before you begin, ensure you have the following:

- Docker installed. Follow the instructions on the [Docker website](https://docs.docker.com/get-docker/) to install Docker
- A Linux environment or Windows with WSL2 for running the scripts. 
    - I recommend using Docker (i know, trippy) in order to run in MacOS or Windows using the `rs-docker.sh` script. Currently working on a better way

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/dpouris/rs-docker
   cd rs-docker
   ```

2. Ensure the scripts have Unix-style line endings. If you cloned the repo on Windows, run:

   ```bash
   sed -i -e 's/\r$//' /app/rs-docker.sh
   ```


## Usage

### Supported commands
---

Currently the only supported command is `run` but in the future I plan to implement most, if not all, of Dockers main commands and some more helper commands.

- run: `rs-docker run <image> <command> <arg1> <arg2> ...`
---

### Using `rs-docker.sh` (Windows/MacOS)

Run the provided script:
```bash
chmod +x ./rs-docker.sh
./rs-docker.sh run ubuntu:latest echo "hello world"
```

### Running the code on system (Linux)

1. Build `src`

   ```bash
   cargo build --release
   ```

2. Run the program using the supported commands:

   ```bash
   ./target/release/rs-docker run ubuntu:latest echo "hello world"
   ```
<!-- ### Building the Image

Build the image using the provided Dockerfile:

```bash
docker build -t custom-container .
```
-->
