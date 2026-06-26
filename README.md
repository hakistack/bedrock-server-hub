# Bedrock Server Manager

**Bedrock Server Manager** is a desktop application built to make Minecraft Bedrock Dedicated Server management easier, especially for users who want to install addons, resource packs, behavior packs, and worlds without manually editing server files.

The goal of this project is to provide a simple, modern, and beginner-friendly manager for Minecraft Bedrock Dedicated Servers while keeping enough power for advanced users.

## Overview

Managing a Bedrock Dedicated Server usually requires working directly with files such as:

* `server.properties`
* `manifest.json`
* `world_behavior_packs.json`
* `world_resource_packs.json`
* `behavior_packs`
* `resource_packs`
* `worlds`

This app aims to simplify that process through a clean desktop interface.

## Planned Features

* Create or import a Bedrock Dedicated Server
* Download official Bedrock Dedicated Server files
* Start, stop, and restart the server
* View live server console logs
* Edit `server.properties` from the UI
* List and manage worlds
* Import `.mcworld` files
* Install `.mcaddon` and `.mcpack` files
* Detect behavior packs and resource packs automatically
* Read addon `manifest.json` metadata
* Apply addons to selected worlds
* Create automatic backups before modifying server files
* Restore backups when needed
* Store local app data using SQLite

## Tech Stack

This project uses:

* [Tauri v2](https://tauri.app/) for the desktop application shell
* [SvelteKit](https://svelte.dev/docs/kit/introduction) for the frontend
* [TypeScript](https://www.typescriptlang.org/) for typed frontend development
* [Rust](https://www.rust-lang.org/) for native filesystem, process, archive, and server logic
* SQLite for local app storage

## Project Structure

```txt
.
├── src/
│   ├── lib/
│   │   ├── components/
│   │   ├── stores/
│   │   ├── api/
│   │   └── types/
│   ├── routes/
│   └── app.html
│
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── core/
│   │   ├── db/
│   │   └── models/
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json
└── README.md
```

## Development Setup

### Requirements

Make sure you have the following installed:

* Node.js
* pnpm, npm, or yarn
* Rust
* Tauri prerequisites for your operating system

For Tauri system dependencies, follow the official setup guide:

```txt
https://tauri.app/start/prerequisites/
```

## Recommended IDE Setup

Recommended extensions for VS Code:

* Svelte for VS Code
* Tauri
* rust-analyzer
* ESLint
* Prettier

## Install Dependencies

Using npm:

```bash
npm install
```

Using pnpm:

```bash
pnpm install
```

## Run in Development

Using npm:

```bash
npm run tauri dev
```

Using pnpm:

```bash
pnpm tauri dev
```

## Build Desktop App

Using npm:

```bash
npm run tauri build
```

Using pnpm:

```bash
pnpm tauri build
```

## Core App Flow

The intended user flow is:

```txt
Create or import server
↓
Configure server settings
↓
Start server
↓
Install addons or worlds
↓
Create automatic backup
↓
Apply changes safely
↓
Restart server if needed
```

## Addon Installation Flow

The addon installation system is planned to work like this:

```txt
Select .mcaddon or .mcpack
↓
Extract package
↓
Read manifest.json
↓
Detect behavior/resource packs
↓
Select target world
↓
Create backup
↓
Copy packs into the server
↓
Update world pack JSON files
↓
Confirm installation
```

## Safety Goals

This app should avoid destructive operations whenever possible.

Before modifying worlds, addon files, or server configuration, the app should create backups automatically. The user should always be able to restore a previous state if an addon installation fails or causes problems.

## Disclaimer

This project is an independent tool and is not affiliated with, endorsed by, or sponsored by Mojang Studios, Microsoft, or Minecraft.

Minecraft and related names are trademarks of their respective owners.

## License

Bedrock Server Manager is free software: you can redistribute it and/or modify
it under the terms of the **GNU General Public License v3.0** as published by the
Free Software Foundation. This program is distributed WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE. See the [LICENSE](./LICENSE) file for the full text.

Copyright (C) 2026 hakistack

> Under the GPL-3.0, any distributed derivative work must also be released under
> the GPL-3.0 with its source code available.
