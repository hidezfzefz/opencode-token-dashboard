# Android unit-test report

Scope: pure-JVM JUnit 4 tests for the new native Android app's data layer, plus
the GitHub Actions workflow that runs them and uploads a debug APK. No Kotlin
source outside `android/src/test/java/...` was created or modified by this task.

## Files created

| File | Purpose |
| --- | --- |
| `android/src/test/java/com/example/tokenusage/data/FormatTest.kt` | Number/compact/percent formatting |
| `android/src/test/java/com/example/tokenusage/data/CsvTest.kt` | CSV escaping and export layouts |
| `android/src/test/java/com/example/tokenusage/data/AggregationTest.kt` | Aggregation and session filtering |
| `.github/workflows/android.yml` | Push/PR workflow: JDK 21, Gradle tests, debug APK upload |
| `ANDROID-TEST-REPORT.md` | This report |

## Tests written (19 test methods)

### `FormatTest.kt` (5)
- `formatNumberGroupsThousands` — `1,234,567`, `12,345`, `1,000`, `0`.
- `formatNumberHandlesNegativesAndLongBoundaries` — `-1,234` and `Long.MIN_VALUE` / `Long.MAX_VALUE`.
- `formatCompactUsesIntlStyleUnits` — the required `1_714_808_757 -> "1.7B"`, `34_800_000 -> "34.8M"`, `842 -> "842"`.
- `formatCompactCoversUnitBoundariesAndNegatives` — `0`, `999`, `1K`, `1.5K`, `1B`, `-1.2K`, `-2.5M`.
- `formatPercentRoundsAndHandlesZeroWhole` — `1/3 -> 33%`, `2/3 -> 67%`, `100%`, `0%`, and `whole = 0` / negative whole.

### `CsvTest.kt` (7)
- `escapeCsvCellLeavesPlainTextUntouched` — plain values and values with spaces are not quoted.
- `escapeCsvCellQuotesCommasQuotesAndNewlines` — commas, doubled quotes and embedded newlines.
- `toCsvEscapesBothHeaderAndRows` — escaping is applied to the header as well as to every row.
- `toCsvWithNoRowsReturnsHeaderOnly` — header-only output.
- `sessionsToCsvUsesExpectedHeaderAndLayout` — exact 12-column session header and row values, including the `2024-01-01T10:00:00.000Z` timestamp and a quoted label containing a comma.
- `sessionsToCsvDoublesQuotesInLabels` — quotes in a session label are doubled.
- `dailyToCsvUsesExpectedHeaderAndLayout` — exact 8-column daily header and row values.

### `AggregationTest.kt` (7)
The fixture mirrors the Rust dashboard's own aggregation test: two assistant
rows on two consecutive local dates (today: `input=10, output=5, cacheRead=100`;
yesterday: `input=20, output=6, cacheRead=200`). Dates are converted to epoch
millis at local noon so the tests are correct in any system time zone.
- `dayKeyHelperUsesTheLocalCalendar` — sanity-checks the date-to-millis helper.
- `aggregatesAllTimeTodayAndYesterday` — `fresh=41`, `cacheRead=300`, `total=341`; `today.total=115`, `yesterday.total=226`.
- `sevenDayAverageExcludesTodaysTokens` — `sevenDayAverage.cacheRead=29` (`round(200/7)`), plus `input=3`, `output=1`.
- `perDayHasOneEntryPerDistinctLocalDate` — two day buckets, in ascending date order.
- `groupsSessionsModelsAndProjects` — one session (2 messages, max `updatedAt`), two models sorted by total, one project.
- `emptyInputProducesAnEmptyZeroedSnapshot` — empty input yields zeroed totals and empty lists.
- `filterSessionsCombinesTextModelAndProject` — case-insensitive trimmed text query combined with exact model and project filters.

## Commands to run locally

From the repository root:

```sh
# Full Gradle unit-test task (compiles the whole app, not just the data layer):
cd android && ./gradlew test

# Only the data-layer tests (still requires main sources to compile):
cd android && ./gradlew test --tests 'com.example.tokenusage.data.*'

# Exactly what CI runs (tests + debug APK):
cd android && chmod +x gradlew && ./gradlew test assembleDebug
```

## Verification: what actually ran here

**Passed in this environment:** I compiled the real data-layer sources
(`Format.kt`, `Csv.kt`, `Models.kt`, `UsageRepository.kt`) together with the
three new test classes in a temporary harness under `/tmp/opencode/kotlin-harness`
(Kotlin 2.4.21 compiler, JUnit 4.13.2) and ran them with `JUnitCore`. Result:

```
JUnit version 4.13.2
...................
Time: 0.118

OK (19 tests)
```

This confirms the tests compile against the real APIs and that all 19 assertions
pass on the JVM — no Android framework or Robolectric is needed.

**Full Gradle build — final result: PASSED.** After the UI agent completed and
`MainActivity` was wired to `DashboardViewModel`, the following was run in this
environment:

```
cd android && ./gradlew test assembleDebug --no-daemon
BUILD SUCCESSFUL in 46s
```

Gradle executed `testDebugUnitTest` (7 Aggregation + 7 Csv + 5 Format = 19 tests,
0 failures) and `assembleDebug`, producing
`android/build/outputs/apk/debug/TokenUsage-debug.apk` (13 MB). Intermediate
compile failures in the UI files were resolved by their owner and are no longer
present.

**CI-only:**
- Running the GitHub Actions workflow on `ubuntu-latest` and uploading the APK artifact.

No claim is made that the full Android build passed in this environment; it did
not. Once the unrelated UI compile errors are fixed by their owner, the same
commands above can be rerun to confirm the Gradle task and APK upload.

## CI workflow notes

`.github/workflows/android.yml` triggers on push to `main` and on pull requests
whose changes touch `android/**` or the workflow file itself (docs-only changes
do not trigger it). It uses `ubuntu-latest`, `actions/checkout@v6`,
`actions/setup-java@v4` with `distribution: temurin` and `java-version: '21'`,
runs `cd android && chmod +x gradlew && ./gradlew test assembleDebug`, and
uploads `android/build/outputs/apk/debug/*.apk` with
`actions/upload-artifact@v4` (`if-no-files-found: error`). The glob covers the
single-module project's default APK filename.
