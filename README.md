# Garcia

### Current supported version: **1.1.22 (Android)**

---

## What is Garcia?

Garcia is a Rust server emulator for the Android version of *ALLfiring*.

---

## Binary release

Download the [latest server](https://github.com/yoncodes/garcia/releases) and [game data](https://archive.org/details/garcia-1.1.22-game-data). Extract both
into one folder and keep their folder structure intact. The result should look
like this:

```text
Garcia/
|-- assets/
|-- data/
|   `-- tables/
|-- config.toml
|-- gameserver.exe
|-- hotpatchserver.exe
|-- muipserver.exe
|-- sdkserver.exe
`-- start-servers.bat
```

1. Edit `config.toml` if the computer's LAN address or ports differ from your
   setup.
2. Use [Garcia Patcher](https://github.com/yoncodes/garcia-patcher) with your
   own original APK or XAPK to point the client at this computer.
3. Run `start-servers.bat`, then launch the patched game.

That is the entire player setup. The game-data archive includes the decoded
tables and patched managed assembly. It does not contain the game or an
APK/XAPK. If the client requests a file missing from the archive,
`hotpatchserver` downloads it from the official CDN and caches it locally.

---

## Building from source

1. Install Rust: [https://rust-lang.org/tools/install](https://rust-lang.org/tools/install)

2. Build the servers:

```text
cargo build --release -p gameserver -p sdkserver -p hotpatchserver -p muipserver
```

The executables are written to `target/release/`.

Copy the four executables, `config.toml`, and `start-servers.bat` into one
folder, then extract the matching game-data archive there.

---

## How to login

Use the game's normal login or register option. The SDK server accepts the 
QuickSDK account ID from the patched client, and the game server
creates the player the first time that account connects.

Player progress is saved in `data/garcia.db`.

---

## Features

- QuickSDK login server
- SQLite support
- Tutorial, feature unlocks, story progress, maps, and interactions
- Heroes, pets, formations, equipment, inventory, profiles, and player roles
- Tasks, missions, mail, shops, gacha, favor, and reward handling
- Dungeon, rogue, rift, boss-rush, trial, and activity state
- Versioned hotpatch cache with offline fallback for previously downloaded files
- MUIP panel for tutorials, feature unlocks, and granting heroes or pets

---

## Known limitations

- Only ALLfiring 1.1.22 is currently supported.
- Some game modes, reward cases may not work yet.
- Most PvE battle simulation happens in the client; Garcia currently focuses
  on account state, progression, rewards, and protocol responses.
- Game tables must come from the same client/hotpatch version being used.
- A hotpatch file must be cached once before it can be served without access
  to the upstream CDN.

---

## MUIP

Start `gameserver` before `muipserver`, then open:

```text
http://127.0.0.1:18890
```

The default token is `garcia`. Change it in `config.toml`.

The panel can complete or reset tutorials, unlock features, and grant missing
heroes or pets. Some changes require the game to reconnect before its UI
refreshes.
