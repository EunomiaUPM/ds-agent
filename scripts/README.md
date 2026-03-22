# Initialization and Automation Scripts Documentation

This document describes all available scripts for initializing, automating, or resetting workflows within the **Eunomia** project.

---

## Table of Contents

- [Auto-Setup](#Auto-Setup)
- [Auto-Start](#Auto-Start)
- [Auto-Onboarding](#Auto-Onboarding)

---

## `Auto-Setup`

### Purpose
Automates the setup of all services.

### Prerequisites
- Docker and Cargo installed
- WaltId operational

### Usage
```bash
    ./auto-setup.sh # for bash
    # or
    ./auto-setup.ps1 # for powershell
```

---

## `Auto-Start`

### Purpose
Automates the startup of all services.

### Prerequisites
- Docker and Cargo installed
- Prior setup completed
- WaltId operational

### Usage
```bash
    ./auto-start.sh # for bash
    # or
    ./auto-start.ps1 # for powershell
```

---

## `Auto-Onboarding`

### Purpose
Automates the full initialization flow of a **Consumer** within a **Provider**, including:

1. Receiving credentials from the Authority.
2. Onboarding the Consumer into the Provider.

### Prerequisites
- Necessary services running (Consumer, Provider, Authority, Wallet)
- `Docker-Compose` operational for databases

### Usage
```bash
    ./auto-onboarding.sh # for bash
    # or 
    ./auto-onboarding.ps1 # for powershell
```
