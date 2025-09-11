<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Guidance for this repo
- Project theme: Local-first Tauri apps replacing common Electron tools.
- Languages: Rust, TypeScript. Prefer Svelte or React for UI.
- Non-goals: No network-by-default; sync is optional and explicit.
- Quality bar: Typed APIs, minimal permissions, small memory and bundle size.
- UX: Single window, tray/shortcuts where appropriate, lazy IPC.
