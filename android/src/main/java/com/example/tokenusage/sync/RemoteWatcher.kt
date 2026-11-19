package com.example.tokenusage.sync

import com.example.tokenusage.data.RemoteRepository
import com.example.tokenusage.data.UsageSnapshot
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.Dispatchers

/**
 * Polls the desktop app over the LAN. Unlike the local file watcher there is no
 * cheap fingerprint to check, so it compares the parsed snapshot and emits only
 * when something actually changed.
 */
class RemoteWatcher(private val host: String) {

    fun events(intervalSeconds: Int): Flow<SyncEvent> = flow {
        val repository = RemoteRepository(host)
        var previous: UsageSnapshot? = null
        var first = true
        while (currentCoroutineContext().isActive) {
            val event = try {
                SyncEvent.Loaded(repository.load())
            } catch (error: Exception) {
                SyncEvent.Failed("Could not reach $host: ${error.message ?: error::class.simpleName}")
            }
            when {
                event is SyncEvent.Loaded && (first || event.snapshot != previous) -> {
                    previous = event.snapshot
                    emit(event)
                }
                event is SyncEvent.Failed && first -> emit(event)
            }
            first = false
            if (intervalSeconds <= 0) break
            delay(intervalSeconds.toLong() * 1_000L)
        }
    }
        .flowOn(Dispatchers.IO)
        .catch { error -> emit(SyncEvent.Failed("Remote sync failed: ${error.message ?: error::class.simpleName}")) }
}
