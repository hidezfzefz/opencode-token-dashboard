# Android app (additional to the Rust desktop app)

This directory contains an **additional native Android app** for viewing OpenCode token usage. It is *not* a replacement for the main app: the Rust desktop application in the repository root stays the primary, cross-platform build. The Android app reuses the same ideas — read-only SQLite access, the same totals/definitions ("fresh" = input + output + reasoning, "total" also includes cache reads and writes, days use the device's local date) — but is written in Kotlin with Jetpack Compose.

It has no network access, no telemetry and no browser/webview runtime.

## Build

A Gradle wrapper is included, so no local Gradle install is needed:

```sh
cd android
./gradlew assembleDebug
```

The APK is produced at `android/app/build/outputs/apk/debug/app-debug.apk`.

## Install

Connect a device (or start an emulator) and run from the repository root:

```sh
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

`adb devices` should list the target first. `-r` replaces an existing install while keeping its data.

## Getting a database onto the device

The app reads a **copy** of your OpenCode database from its own private files directory:

```
/data/data/com.example.tokenusage/files/token-usage/opencode.db
```

Push your local database there. From the folder containing `opencode.db`:

```sh
adb shell mkdir -p /data/data/com.example.tokenusage/files/token-usage
adb push opencode.db /data/data/com.example.tokenusage/files/token-usage/opencode.db
```

The directory must already exist when the app is installed — install the APK before pushing.

On a physical (non-emulator) device, writing directly to `/data/data/...` usually needs root:

```sh
adb root
adb remount   # only if prompted
```

If `adb root` is unavailable, use `run-as` for this debuggable build, which works without root:

```sh
adb shell mkdir -p /data/data/com.example.tokenusage/files/token-usage
adb push opencode.db /data/local/tmp/opencode.db
adb shell run-as com.example.tokenusage cp /data/local/tmp/opencode.db \
  files/token-usage/opencode.db
adb shell rm /data/local/tmp/opencode.db
```

The file must be readable by the app user; `run-as` copies it into the app's own storage, which guarantees that.

### The database is opened read-only

`UsageRepository.load` opens the file with `SQLiteDatabase.OPEN_READ_ONLY`. The app never inserts, updates, deletes or migrates anything. If OpenCode is running on the same device it may hold the database in WAL mode; copy the `.db` file (and, if present, its `-wal` / `-shm` siblings) together for a consistent snapshot.

## Auto sync and settings

Preferences are persisted with `androidx.datastore.preferences` (file `token-usage-settings`), so they survive restarts:

| Setting | Values | Default |
|---|---|---|
| Auto sync interval | Off (0), 1, 5, 15 or 60 seconds | 5 seconds |
| Show all days | on / off | off |
| Desktop host | `host[:port]` or blank | blank (local database file) |

Unsupported stored values are normalized back to the default on read, so a stale or corrupted preferences file cannot break the UI.

Auto sync is a coroutine watcher. On every tick it computes a cheap fingerprint — the length and last-modified time of `opencode.db` plus its `-wal` and `-shm` siblings — and only re-queries SQLite when that fingerprint changes. "Off" still performs one initial load, then stops watching. The **Refresh** button always forces a fresh load. Any load error is surfaced as a readable message in the UI instead of crashing the app.

## Signing and updates

CI builds are signed with the stable debug keystore committed at `keystore/debug.jks`, so every release has the same signature and normal updates work. `versionCode` is bumped for each release.

Releases before v0.3.2 used a fresh runner-generated debug key. If you installed one of those, Android will refuse to update it over v0.3.2 with a "different signature" error — uninstall that old copy once, then install v0.3.2. From then on updates will install normally.

## Live tracking from the desktop app

Android apps cannot read another app's private database. To follow live usage instead of pushing DB copies, run the Rust dashboard on the machine that has OpenCode and give the phone its LAN address:

```sh
opencode-token-dashboard --web --host 0.0.0.0 --port 3000
```

Enter `http://192.168.x.x:3000` in the **Desktop host** field. The app then polls `GET /api/usage` instead of reading a local file; the same auto-sync interval applies and CSV exports still come from the last loaded snapshot. Leave the field blank to return to local-file mode.

The endpoint is read-only and unauthenticated. Only use this on a trusted LAN; do not expose the port to the internet. The manifest enables `INTERNET` and cleartext HTTP for this local LAN use.

## CSV export

Exports are written with no permission prompt and no Storage Access Framework picker, into the app's own external files directory:

```
/storage/emulated/0/Android/data/com.example.tokenusage/files/exports/
```

- `opencode-token-usage-sessions.csv`
- `opencode-token-usage-daily.csv`

You can pull them off the device with:

```sh
adb pull /sdcard/Android/data/com.example.tokenusage/files/exports/opencode-token-usage-sessions.csv .
```

Each export returns the written `File` (or a readable failure message), which the UI shows as a Toast/status line. The files are removed when the app is uninstalled.

## Code map (wiring owned by this layer)

- `settings/SettingsRepository.kt` — `Settings` data class, `Flow<Settings>`, `setAutoSyncSeconds` / `setAllDays` suspend setters. `syncOptions` provides the labelled choices for a spinner.
- `sync/AutoSync.kt` — `AutoSync` fingerprint watcher (`snapshots(intervalSeconds)`, `loadOnce()`), plus `DashboardViewModel` exposing `StateFlow<UiState>` (`snapshot`, `isLoading`, `error`, `exportStatus`), a `settings` flow, `refresh()`, the two `set...` methods and `exportSessions()` / `exportDaily()`. Construct it with `DashboardViewModel.factory(context)` in `viewModel(...)`.
- `export/CsvExporter.kt` — `CsvExporter(context)`, `exportSessions(rows)` / `exportDaily(rows)` returning `CsvExportResult.Success(file)` or `CsvExportResult.Failure(message)`.
- `data/` — the shared models, aggregation, formatting and CSV serializers (`sessionsToCsv` / `dailyToCsv`) provided by the data layer.

`DashboardViewModel` uses `viewModelScope`, so `androidx.lifecycle:lifecycle-viewmodel-ktx` is required on the classpath in addition to `lifecycle-runtime-ktx`.
