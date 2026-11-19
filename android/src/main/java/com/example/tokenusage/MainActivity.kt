package com.example.tokenusage

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.tokenusage.data.UsageSnapshot
import com.example.tokenusage.sync.DashboardViewModel
import com.example.tokenusage.ui.TokenUsageApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val context = LocalContext.current
            val viewModel: DashboardViewModel = viewModel(
                factory = DashboardViewModel.factory(context.applicationContext),
            )
            val uiState by viewModel.uiState.collectAsState()
            val settings by viewModel.settings.collectAsState()

            TokenUsageMainContent(
                snapshot = uiState.snapshot,
                error = uiState.error,
                refreshing = uiState.isLoading,
                onRefresh = viewModel::refresh,
                onExportSessions = viewModel::exportSessions,
                onExportDaily = viewModel::exportDaily,
                autoSyncSeconds = settings.autoSyncSeconds,
                onAutoSyncChange = viewModel::setAutoSyncSeconds,
                allDays = settings.allDays,
                onAllDaysChange = viewModel::setAllDays,
                exportStatus = uiState.exportStatus,
                remoteUrl = settings.remoteUrl,
                onRemoteUrlChange = viewModel::setRemoteUrl,
            )
        }
    }
}

@Composable
fun TokenUsageMainContent(
    snapshot: UsageSnapshot? = null,
    error: String? = null,
    onRefresh: () -> Unit = {},
    onExportSessions: () -> Unit = {},
    onExportDaily: () -> Unit = {},
    refreshing: Boolean = false,
    autoSyncSeconds: Int = 5,
    onAutoSyncChange: (Int) -> Unit = {},
    allDays: Boolean = false,
    onAllDaysChange: (Boolean) -> Unit = {},
    exportStatus: String? = null,
    remoteUrl: String = "",
    onRemoteUrlChange: (String) -> Unit = {},
) {
    TokenUsageApp(
        snapshot = snapshot,
        error = error,
        onRefresh = onRefresh,
        onExportSessions = onExportSessions,
        onExportDaily = onExportDaily,
        refreshing = refreshing,
        autoSyncSeconds = autoSyncSeconds,
        allDays = allDays,
        onAutoSyncChange = onAutoSyncChange,
        onAllDaysChange = onAllDaysChange,
        exportStatus = exportStatus,
        remoteUrl = remoteUrl,
        onRemoteUrlChange = onRemoteUrlChange,
        modifier = Modifier.fillMaxSize(),
    )
}
