# Android app

`android/` contains a native Android app (Kotlin + Jetpack Compose + Material 3) that
surfaces the same token-usage data as the Rust app. It is **additional to** the Rust
application in `src/` — the Rust app is unchanged and remains the primary desktop /
CLI experience. The Android app is a single Gradle module (`com.example.tokenusage`).

The UI uses the same dark, calm palette as the web view: black background, near-black
cards, off-white text, muted secondary text and hairline dividers (defined in
`android/src/main/res/values/colors.xml`).

## Requirements

- JDK 21
- Android SDK with platform 37 and build tools 36+ (path set in `android/local.properties`)
- No need to set `ANDROID_HOME`; `local.properties` points at `/home/user/Android/Sdk`

`local.properties` is machine-specific and gitignored. The Gradle wrapper is
checked in, so a system Gradle install is not required.

## Build and install

From the repository root:

```sh
cd android
./gradlew tasks                 # verify configuration
./gradlew assembleDebug        # build android/build/outputs/apk/debug/app-debug.apk
./gradlew installDebug         # build and install on a connected device/emulator
```

Unit tests:

```sh
cd android
./gradlew testDebugUnitTest
```

To point the project at a different SDK, edit `android/local.properties`:

```properties
sdk.dir=/path/to/Android/Sdk
```
