# AI IPC — Cursor / VS Code extension

Two ways to use it: **install a package** (use it like any extension), or **run from source** (for development).

---

## Option A — Install the `.vsix` (normal use)

You need Node.js and this repo (or a copy of the `collab-vscode` folder).

1. **Build the package** (from your machine, in a terminal):

   ```bash
   cd collab-vscode
   npm install
   npm run compile
   npx --yes @vscode/vsce package
   ```

   This creates a file named like **`collab-vscode-0.2.0.vsix`** in `collab-vscode/`.

2. **Install it in Cursor or VS Code**

   - Open the **Extensions** view (square icon in the sidebar, or `Cmd+Shift+X` / `Ctrl+Shift+X`).
   - Click the **`⋯`** menu at the top of the Extensions panel.
   - Choose **Install from VSIX…**
   - Pick the **`collab-vscode-0.2.0.vsix`** file.
   - When prompted, **Reload** the window.

3. **Configure** (Settings → search **AI IPC** or **collab**):

   | Setting | Meaning |
   |--------|---------|
   | `collab.server` | Your server URL, e.g. `http://localhost:8000` |
   | `collab.token` | Same secret as `COLLAB_TOKEN` on the server |
   | `collab.instance` | Your name on the roster (e.g. `my-laptop`) |

   You can also set `COLLAB_SERVER`, `COLLAB_TOKEN`, and `COLLAB_INSTANCE` in the environment, or put them in a **`.env`** file in your workspace root.

4. **Use it**: Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`) → type **AI IPC**, **ipc**, or **collab** → e.g. **AI IPC: Open Chat**. The **AI IPC** icon also appears on the activity bar.

---

## Option B — Run from source (extension developers only)

Use this when you’re changing the code and want to debug.

1. Open the **`collab-vscode`** folder (or the **repo root** if you use the root `.vscode` launch config) in Cursor/VS Code.

2. **Run → Start Debugging** (`F5`). A **second** window opens — the **Extension Development Host**. The extension only runs in **that** window, not in your first window.

3. In the **second** window, use the palette and sidebar as above.

To try the extension in your **main** window without F5, use **Option A** (VSIX).

---

## Requirements

- **Server**: `collab-server` running and reachable, with the same **token** you put in settings.
- **Engine**: VS Code **1.85+** or a recent **Cursor** (VS Code–compatible).

Full API and behavior: [`SPEC.md`](SPEC.md).
