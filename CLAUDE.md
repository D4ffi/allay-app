# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is "Allay"—a dedicated Minecraft server management application built with Tauri (Rust backend + React/TypeScript frontend). The project uses Vite for frontend tooling and TailwindCSS for styling.

## Development Commands

### Frontend Development
- `npm run dev` - Start Vite development server (frontend only)
- `npm run build` - Build TypeScript and create a production bundle
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
      - `AllayLayout.tsx` - Main application layout wrapper with window controls
      - `ActionBar.tsx` - Expandable action bar with modal integration
      - `ToolTip.tsx` - Reusable tooltip component with positioning options
      - `Dropdown.tsx` - Custom dropdown with ChevronDown icon
      - `RadioGroup.tsx` - Flexible radio button group component
      - `ChangeServerImg.tsx` - Image selector with file validation
      - `Modal.tsx` - Generic modal component for all modal needs
    - `/modals/` - Specific modal implementations
      - `CreateServerModal.tsx` - Server creation modal with complete workflow
  - `/hooks/` - Custom React hooks
    - `useWindowControls.ts` - Window control functions (minimize, maximize, close)
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
- **Backend**: Rust with a Tauri framework
- **Bundling**: Vite (frontend), Tauri (application packaging)
- **Styling**: TailwindCSS with Vite plugin

### Development Configuration
- Frontend dev server runs on port 1420
- HMR (Hot Module Reload) on port 1421 when using a custom host
- Tauri watches for frontend changes and rebuilds accordingly
- TypeScript strict mode is enabled with comprehensive linting rules

## Components

### ActionBar
- Expandable action bar component located in the top-left corner
- Contains animated buttons for Create, Search, Pin, Filter, and Settings actions
- Features smooth expand/collapse animation with configurable timing
- Includes tooltips for each action button
- Uses `pointer-events-none` to prevent interaction with hidden buttons
- Integrates with a Modal system for server creation workflow

### ToolTip
- Reusable tooltip component with multiple positioning options (top, bottom, left, right)
- Configurable delay before showing/hiding
- Automatically hides when content changes to prevent display issues
- Uses absolute positioning with transform utilities for precise placement
- Used throughout UI for enhanced accessibility

### Modal System
- **Modal.tsx** - Generic modal component for all modal needs
- Responsive design with configurable sizes (sm, md, lg, xl)
- Overlay with click-outside-to-close functionality
- Smooth animations with fade-in and zoom effects
- Header with title and close button with tooltip
- Completely reusable for any modal content

### Form Components

#### Dropdown
- Custom dropdown component with ChevronDown icon
- Supports keyboard navigation and accessibility
- Click-outside-to-close functionality
- Multiple size options and error handling
- Smooth animations for open/close states

#### RadioGroup
- Flexible radio button group component
- Multiple layout options: vertical, horizontal, grid
- Visual feedback with hover and selected states
- Support for descriptions on each option
- Custom styled radio buttons with animations

#### ChangeServerImg
- Image selector component with default profile.png
- Click-to-change functionality (no separate button needed)
- File validation (image types only, max 5MB)
- Preview functionality with URL.createObjectURL
- Reset button with stopPropagation to prevent conflicts
- Hover overlay with the "Click to change" message

### Server Creation Modal
- Complete workflow for Minecraft server creation
- **Server Icon**: ChangeServerImg component for custom server images
- **Server Name**: Text input field
- **Minecraft Version**: Dropdown with 10+ versions (1.21.1 to 1.16.5)
- **Mod Loader Selection**: RadioGroup with six options:
  - Vanilla (pure Minecraft)
  - Fabric (lightweight modding)
  - Forge (most popular modding platform)
  - NeoForge (modern Forge fork)
  - Paper (high-performance server software)
  - Quilt (community-driven Fabric fork)
- **Mod Loader Version**: Conditional dropdown that appears only when:
  - Minecraft version is selected AND
  - Non-vanilla mod loader is selected
- **Action Buttons**: Done/Cancel with proper state management

### Modal State Management
- Auto-reset functionality: all form fields reset when modal closes
- Conditional UI: mod loader versions only show when relevant
- Form validation and error handling
- State synchronization between parent and child components

## Technical Implementation Details

### Animation System
- Consistent timing across components (200–300 ms transitions)
- Staggered animations in ActionBar for a professional feel
- Hover effects with smooth scaling and color transitions
- Focus rings for accessibility compliance

### Event Handling
- `stopPropagation()` used to prevent event conflicts
- Click-outside handlers for dropdowns and modals
- Keyboard navigation support in form components
- Touch events support for mobile compatibility

### Data Structures
- Minecraft versions with value/label pairs
- Mod loader options with descriptions
- Version mappings for each mod loader type
- File handling for image uploads with validation

### Styling Patterns
- Consistent color scheme (grays, blues for primary actions)
- Tailwind CSS utility classes for rapid development
- Responsive design principles
- Consistent spacing and typography scale

## Component Relationships

### AllayLayout Integration
- Contains window controls (minimize, maximize, close) with tooltips
- Uses `useWindowControls` hook for Tauri window management
- Includes a drag area for window movement
- Houses application logo and title

### ActionBar → Modal Flow
1. ActionBar contains expandable buttons with tooltips
2. "Create" button opens CreateServerModal component
3. CreateServerModal uses a generic Modal component as base
4. Form state management is self-contained within CreateServerModal
5. Auto-reset functionality on modal close/submitting
6. Clean separation of concerns between UI trigger and modal logic

### Form Component Chain
1. ChangeServerImg → File selection and preview
2. Text input → Server name
3. Dropdown → Minecraft version selection
4. RadioGroup → Mod loader type selection
5. Conditional Dropdown → Mod loader version (only if needed)
6. Action buttons → Form submission

## Development Patterns

### State Management
- Local state with useState for form data
- Custom hooks for complex logic (useWindowControls)
- Event handlers with proper cleanup (useEffect)
- Conditional rendering based on form state

### File Organization
- **Components grouped by functionality**: `/common`, `/modals`, `/hooks`
- **Reusable components in `/common`**: Modal, Dropdown, RadioGroup, etc.
- **Specific modal implementations in `/modals`**: CreateServerModal, future modals
- **Custom hooks in `/hooks`**: useWindowControls, future custom logic
- **Clean separation**: UI logic vs business logic vs presentation components

### Accessibility Features
- Tooltips throughout the interface
- Focus rings on interactive elements
- Keyboard navigation support
- ARIA labels where needed
- Proper semantic HTML structure

## Key Files
- `package.json` - Frontend dependencies and npm scripts
- `src-tauri/Cargo.toml` - Rust dependencies (lib name: `allay_app_lib`)
- `src-tauri/tauri.conf.json` - App configuration including custom icons
- `vite.config.ts` - Vite configuration optimized for Tauri development
- `tsconfig.json` - TypeScript configuration with strict settings
- `CLAUDE.md` - Project documentation and development guidelines