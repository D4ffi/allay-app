# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is "Allay" - a dedicated Minecraft server management application built with Tauri (Rust backend + React/TypeScript frontend). The project uses Vite for frontend tooling and TailwindCSS for styling.

## Development Commands

### Frontend Development
- `npm run dev` - Start Vite development server (frontend only)
- `npm run build` - Build TypeScript and create production bundle
- `npm run preview` - Preview production build

### Tauri Development
- `npm run tauri dev` - Start full Tauri development mode (launches both frontend and Tauri app)
- `npm run tauri build` - Build production Tauri application for distribution

### Rust Backend
- `cargo build` (in src-tauri/) - Build Rust backend
- `cargo run` (in src-tauri/) - Run Rust backend directly

## Architecture

### Project Structure
- `/src/` - React/TypeScript frontend code
  - `/components/` - Reusable React components
    - `/common/` - Shared UI components
      - `AllayLayout.tsx` - Main application layout wrapper
      - `ActionBar.tsx` - Expandable action bar with animated buttons
      - `ToolTip.tsx` - Reusable tooltip component with positioning options
  - `/pages/` - Page-level components (currently has Home.tsx)
  - `App.tsx` - Main application component
  - `main.tsx` - React app entry point
- `/src-tauri/` - Rust backend code
  - `/src/lib.rs` - Main Tauri application logic and command handlers
  - `/src/main.rs` - Entry point that calls lib.rs
  - `tauri.conf.json` - Tauri application configuration
  - `Cargo.toml` - Rust dependencies and project metadata

### Frontend-Backend Communication
- Uses Tauri's `invoke()` API to call Rust commands from React
- Commands are defined in `src-tauri/src/lib.rs` with `#[tauri::command]` attribute
- Example: `greet` command takes a name string and returns a formatted greeting

### Technology Stack
- **Frontend**: React 18, TypeScript, TailwindCSS, Vite
- **Backend**: Rust with Tauri framework
- **Bundling**: Vite (frontend), Tauri (application packaging)
- **Styling**: TailwindCSS with Vite plugin

### Development Configuration
- Frontend dev server runs on port 1420
- HMR (Hot Module Reload) on port 1421 when using custom host
- Tauri watches for frontend changes and rebuilds accordingly
- TypeScript strict mode enabled with comprehensive linting rules

## Components

### ActionBar
- Expandable action bar component located in top-left corner
- Contains animated buttons for Create, Search, Pin, Filter, and Settings actions
- Features smooth expand/collapse animation with configurable timing
- Includes tooltips for each action button
- Uses `pointer-events-none` to prevent interaction with hidden buttons

### ToolTip
- Reusable tooltip component with multiple positioning options (top, bottom, left, right)
- Configurable delay before showing/hiding
- Automatically hides when content changes to prevent display issues
- Uses absolute positioning with transform utilities for precise placement

## Key Files
- `package.json` - Frontend dependencies and npm scripts
- `src-tauri/Cargo.toml` - Rust dependencies (lib name: `allay_app_lib`)
- `vite.config.ts` - Vite configuration optimized for Tauri development
- `tsconfig.json` - TypeScript configuration with strict settings