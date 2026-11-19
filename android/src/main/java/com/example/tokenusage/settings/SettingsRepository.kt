package com.example.tokenusage.settings

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map

/**
 * The small set of preferences that survive process restarts.
 *
 * [autoSyncSeconds] is the interval, in seconds, at which the dashboard checks
 * the database for changes. `0` means auto sync is off; an explicit initial
 * load still happens. Invalid values are normalized back to the default when
 * read, so a corrupted/old preferences file can never break the UI.
 */
data class Settings(
    val autoSyncSeconds: Int = DEFAULT_AUTO_SYNC_SECONDS,
    val allDays: Boolean = DEFAULT_ALL_DAYS,
    val remoteUrl: String = DEFAULT_REMOTE_URL,
) {
    companion object {
        const val DEFAULT_AUTO_SYNC_SECONDS: Int = 5
        const val DEFAULT_ALL_DAYS: Boolean = false
        const val DEFAULT_REMOTE_URL: String = ""

        /** Mirrors the desktop app's Off / 1 / 5 / 15 / 60 second choices. */
        val ALLOWED_AUTO_SYNC_SECONDS: Set<Int> = setOf(0, 1, 5, 15, 60)

        val SYNC_OPTIONS: List<Pair<Int, String>> = syncOptions
    }
}

/** Human readable choices for the sync-interval setting. */
val syncOptions: List<Pair<Int, String>> = listOf(
    0 to "Off",
    1 to "1 second",
    5 to "5 seconds",
    15 to "15 seconds",
    60 to "60 seconds",
)

/** Maps any unsupported interval to the documented default. */
fun normalizeAutoSyncSeconds(seconds: Int): Int =
    if (seconds in Settings.ALLOWED_AUTO_SYNC_SECONDS) seconds else Settings.DEFAULT_AUTO_SYNC_SECONDS

private object PreferenceKeys {
    val AUTO_SYNC_SECONDS = intPreferencesKey("auto_sync_seconds")
    val ALL_DAYS = booleanPreferencesKey("all_days")
    val REMOTE_URL = stringPreferencesKey("remote_url")
}

// DataStore delegates must be declared at top level so the single instance is
// shared for the whole process.
private val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(
    name = "token-usage-settings",
)

/**
 * Persists [Settings] with androidx.datastore.preferences.
 *
 * Pass an application context; the repository does not hold on to an Activity.
 */
class SettingsRepository(private val context: Context) {

    /** Emits the current settings and re-emits whenever a value changes. */
    val settings: Flow<Settings> = context.settingsDataStore.data
        .map { preferences ->
            Settings(
                autoSyncSeconds = normalizeAutoSyncSeconds(
                    preferences[PreferenceKeys.AUTO_SYNC_SECONDS]
                        ?: Settings.DEFAULT_AUTO_SYNC_SECONDS,
                ),
                allDays = preferences[PreferenceKeys.ALL_DAYS] ?: Settings.DEFAULT_ALL_DAYS,
                remoteUrl = preferences[PreferenceKeys.REMOTE_URL] ?: Settings.DEFAULT_REMOTE_URL,
            )
        }
        .catch {
            // A broken preferences file should fall back to defaults, not crash.
            emit(Settings())
        }

    /** Stores a valid interval; unsupported values are normalized first. */
    suspend fun setAutoSyncSeconds(seconds: Int) {
        val normalized = normalizeAutoSyncSeconds(seconds)
        context.settingsDataStore.edit { preferences ->
            preferences[PreferenceKeys.AUTO_SYNC_SECONDS] = normalized
        }
    }

    suspend fun setAllDays(enabled: Boolean) {
        context.settingsDataStore.edit { preferences ->
            preferences[PreferenceKeys.ALL_DAYS] = enabled
        }
    }

    /** Blank means local database mode; otherwise host[:port] of the desktop app. */
    suspend fun setRemoteUrl(url: String) {
        context.settingsDataStore.edit { preferences ->
            preferences[PreferenceKeys.REMOTE_URL] = url.trim()
        }
    }
}
