package com.example.tokenusage.sync

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.example.tokenusage.data.UsageRepository
import com.example.tokenusage.data.UsageSnapshot
import com.example.tokenusage.export.CsvExporter
import com.example.tokenusage.export.CsvExportResult
import com.example.tokenusage.settings.Settings
import com.example.tokenusage.settings.SettingsRepository
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/** Supplies a fresh snapshot for a given database file. */
fun interface SnapshotLoader {
    fun load(dbFile: File): UsageSnapshot
}

/** Result of one load attempt. Errors are data, never thrown at the caller. */
sealed interface SyncEvent {
    data class Loaded(val snapshot: UsageSnapshot) : SyncEvent
    data class Failed(val message: String) : SyncEvent
}

/**
 * A cheap, comparable fingerprint of the database and its WAL siblings.
 * OpenCode writes most changes to the `-wal` file, so all three files count.
 */
data class Fingerprint(val stamps: List<Stamp>) {
    data class Stamp(val path: String, val length: Long, val lastModified: Long)

    companion object {
        val EMPTY: Fingerprint = Fingerprint(emptyList())
    }
}

private fun describe(error: Throwable): String =
    error.message?.takeIf { it.isNotBlank() } ?: error::class.simpleName ?: "Unknown error"

/**
 * Watches the database file and reloads a snapshot only when its fingerprint
 * changes. Cheap: it stats at most three files per tick and does no SQLite work
 * while the database is unchanged.
 *
 * An interval of `0` (Off) performs one initial load and then stops.
 */
class AutoSync(
    val dbFile: File,
    private val loader: SnapshotLoader,
) {

    /** Current cheap fingerprint; suitable for manual change detection too. */
    fun fingerprint(): Fingerprint = fingerprintOf(dbFile)

    /** Loads once, regardless of whether the fingerprint changed. */
    suspend fun loadOnce(): SyncEvent = loadSafely()

    /**
     * Emits a [SyncEvent] immediately and then whenever the database changes,
     * waking up at most every [intervalSeconds] seconds.
     */
    fun snapshots(intervalSeconds: Int): Flow<SyncEvent> = flow {
        var previous: Fingerprint? = null
        var first = true
        while (currentCoroutineContext().isActive) {
            val current = fingerprint()
            if (first || current != previous) {
                previous = current
                first = false
                emit(loadSafely())
            }
            if (intervalSeconds <= 0) break // Off: one initial load, then stop.
            delay(intervalSeconds.toLong() * 1_000L)
        }
    }
        .flowOn(Dispatchers.IO)
        .catch { error -> emit(SyncEvent.Failed("Auto sync failed: ${describe(error)}")) }

    /** Convenience for callers that only care about successfully loaded data. */
    fun usageSnapshots(intervalSeconds: Int): Flow<UsageSnapshot> =
        snapshots(intervalSeconds).mapNotNull { (it as? SyncEvent.Loaded)?.snapshot }

    private suspend fun loadSafely(): SyncEvent = withContext(Dispatchers.IO) {
        try {
            SyncEvent.Loaded(loader.load(dbFile))
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (error: Exception) {
            SyncEvent.Failed("Could not load database: ${describe(error)}")
        }
    }

    companion object {
        /** length + lastModified of the .db, -wal and -shm files that exist. */
        fun fingerprintOf(dbFile: File): Fingerprint {
            val path = dbFile.absolutePath
            val candidates = listOf(dbFile, File("$path-wal"), File("$path-shm"))
            val stamps = candidates.mapNotNull { candidate ->
                runCatching {
                    if (!candidate.exists()) null
                    else Fingerprint.Stamp(candidate.absolutePath, candidate.length(), candidate.lastModified())
                }.getOrNull()
            }
            return Fingerprint(stamps)
        }
    }
}

/** Immutable view of everything the screen needs. */
data class UiState(
    val snapshot: UsageSnapshot? = null,
    val isLoading: Boolean = false,
    val error: String? = null,
    val exportStatus: String? = null,
)

/**
 * Owns the snapshot, settings and export status for the Compose UI.
 *
 * Construction from a [Context] is the common path; the collaborator parameters
 * have defaults but can be supplied to drive the class from tests.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class DashboardViewModel(
    context: Context,
    private val settingsRepository: SettingsRepository = SettingsRepository(context.applicationContext),
    private val exporter: CsvExporter = CsvExporter(context.applicationContext),
    private val autoSync: AutoSync = AutoSync(
        UsageRepository.databaseFile(context.applicationContext),
        SnapshotLoader { file -> UsageRepository.load(file) },
    ),
) : ViewModel() {

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    private val _settings = MutableStateFlow(Settings())
    val settings: StateFlow<Settings> = _settings.asStateFlow()

    init {
        // Keep the settings state current, and restart the watcher whenever the
        // configured interval changes. flatMapLatest cancels the previous flow.
        viewModelScope.launch {
            settingsRepository.settings.collect { value -> _settings.value = value }
        }
        viewModelScope.launch {
            _settings
                .map { Pair(it.autoSyncSeconds, it.remoteUrl) }
                .distinctUntilChanged()
                .flatMapLatest { (seconds, remoteUrl) ->
                    if (remoteUrl.isBlank()) {
                        autoSync.snapshots(seconds)
                    } else {
                        RemoteWatcher(remoteUrl).events(seconds)
                    }
                }
                .collect { event -> updateState { state -> state.apply(event) } }
        }
    }

    /** Explicit refresh (e.g. the Refresh button), independent of auto sync. */
    fun refresh() {
        viewModelScope.launch {
            updateState { it.copy(isLoading = true, error = null) }
            val remoteUrl = _settings.value.remoteUrl
            val event = if (remoteUrl.isBlank()) {
                autoSync.loadOnce()
            } else {
                RemoteWatcher(remoteUrl).events(0).let { flow ->
                    var result: SyncEvent? = null
                    flow.collect { result = it }
                    result ?: SyncEvent.Failed("No response from $remoteUrl")
                }
            }
            updateState { state ->
                when (event) {
                    is SyncEvent.Loaded -> state.copy(
                        snapshot = event.snapshot,
                        isLoading = false,
                        error = null,
                    )
                    is SyncEvent.Failed -> state.copy(isLoading = false, error = event.message)
                }
            }
        }
    }

    fun setAutoSyncSeconds(seconds: Int) {
        viewModelScope.launch { settingsRepository.setAutoSyncSeconds(seconds) }
    }

    fun setAllDays(enabled: Boolean) {
        viewModelScope.launch { settingsRepository.setAllDays(enabled) }
    }

    fun setRemoteUrl(url: String) {
        viewModelScope.launch { settingsRepository.setRemoteUrl(url) }
    }

    /** Export triggers; results are published through [uiState].exportStatus. */
    fun exportSessions() = export(ExportKind.SESSIONS)

    fun exportDaily() = export(ExportKind.DAILY)

    fun clearExportStatus() = updateState { it.copy(exportStatus = null) }

    fun clearError() = updateState { it.copy(error = null) }

    private enum class ExportKind { SESSIONS, DAILY }

    private fun export(kind: ExportKind) {
        viewModelScope.launch {
            val snapshot = _uiState.value.snapshot
            if (snapshot == null) {
                updateState { it.copy(exportStatus = "Load a database before exporting.") }
                return@launch
            }
            val result = withContext(Dispatchers.IO) {
                when (kind) {
                    ExportKind.SESSIONS -> exporter.exportSessions(snapshot.sessions)
                    ExportKind.DAILY -> exporter.exportDaily(snapshot.perDay)
                }
            }
            updateState { it.copy(exportStatus = CsvExporter.message(result)) }
        }
    }

    private fun updateState(transform: (UiState) -> UiState) {
        _uiState.value = transform(_uiState.value)
    }

    private fun UiState.apply(event: SyncEvent): UiState = when (event) {
        is SyncEvent.Loaded -> copy(snapshot = event.snapshot, isLoading = false, error = null)
        is SyncEvent.Failed -> copy(isLoading = false, error = event.message)
    }

    companion object {
        /** Factory for `viewModel(DashboardViewModel.factory(context))`. */
        fun factory(context: Context): ViewModelProvider.Factory = Factory(context)

        class Factory(private val context: Context) : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T {
                if (modelClass.isAssignableFrom(DashboardViewModel::class.java)) {
                    return DashboardViewModel(context.applicationContext) as T
                }
                throw IllegalArgumentException("Unknown ViewModel class: ${modelClass.name}")
            }
        }
    }
}
