package com.example.tokenusage.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.unit.dp

/** The four dashboard pages. */
enum class DashboardTab(val label: String) {
    Overview("Overview"),
    Sessions("Sessions"),
    Models("Models"),
    Projects("Projects"),
}

/**
 * Dark top bar: wordmark on the left, refresh on the right and four pill
 * tabs underneath, mirroring `.topbar` / `.nav-link` in the web stylesheet.
 */
@Composable
internal fun TopBar(
    selectedTab: DashboardTab,
    onTabSelected: (DashboardTab) -> Unit,
    onRefresh: () -> Unit,
    modifier: Modifier = Modifier,
    refreshing: Boolean = false,
    autoSyncSeconds: Int = 5,
    onAutoSyncChange: (Int) -> Unit = {},
    allDays: Boolean = false,
    onAllDaysChange: (Boolean) -> Unit = {},
    remoteUrl: String = "",
    onRemoteUrlChange: (String) -> Unit = {},
) {
    Column(modifier = modifier.background(Palette.Background)) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Wordmark()
            RefreshButton(refreshing = refreshing, onClick = onRefresh)
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            DashboardTab.entries.forEach { tab ->
                NavPill(
                    label = tab.label,
                    selected = tab == selectedTab,
                    onClick = { onTabSelected(tab) },
                    modifier = Modifier.weight(1f),
                )
            }
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(20.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val syncLabel = if (autoSyncSeconds == 0) "Off" else "$autoSyncSeconds s"
            Text(
                text = "Auto sync: $syncLabel",
                color = Palette.Muted,
                style = androidx.compose.material3.MaterialTheme.typography.labelMedium,
                modifier = Modifier.clickable {
                    val options = listOf(0, 1, 5, 15, 60)
                    val next = options[(options.indexOf(autoSyncSeconds) + 1) % options.size]
                    onAutoSyncChange(next)
                },
            )
            Text(
                text = if (allDays) "Range: All time" else "Range: Last 30 days",
                color = Palette.Muted,
                style = androidx.compose.material3.MaterialTheme.typography.labelMedium,
                modifier = Modifier.clickable { onAllDaysChange(!allDays) },
            )
            TextField(
                value = remoteUrl,
                onValueChange = onRemoteUrlChange,
                label = { Text("Desktop host (blank = local DB)") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
        }

        Hairline()
    }
}

@Composable
private fun Wordmark() {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(26.dp)
                .border(1.dp, Palette.Hairline, RoundedCornerShape(8.dp))
                .background(Palette.Raised, RoundedCornerShape(8.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Canvas(modifier = Modifier.size(14.dp)) {
                val stroke = 1.4.dp.toPx()
                val points = listOf(
                    Offset(0f, size.height * 0.7f),
                    Offset(size.width * 0.3f, size.height * 0.35f),
                    Offset(size.width * 0.55f, size.height * 0.6f),
                    Offset(size.width, size.height * 0.15f),
                )
                for (i in 0 until points.size - 1) {
                    drawLine(
                        color = Palette.Secondary,
                        start = points[i],
                        end = points[i + 1],
                        strokeWidth = stroke,
                        cap = StrokeCap.Round,
                    )
                }
            }
        }
        Text(
            text = "token usage",
            style = DashboardTypography.bodyLarge,
            color = Palette.Text,
            modifier = Modifier.padding(start = 8.dp),
        )
    }
}

@Composable
private fun NavPill(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = modifier
            .background(if (selected) Palette.Raised else Palette.Background, shape)
            .clickable(onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 8.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = DashboardTypography.labelMedium,
            color = if (selected) Palette.Text else Palette.Muted,
            maxLines = 1,
        )
    }
}

@Composable
private fun RefreshButton(
    onClick: () -> Unit,
    refreshing: Boolean,
) {
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .background(Palette.Raised, shape)
            .border(1.dp, Palette.Hairline, shape)
            .clickable(enabled = !refreshing, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 7.dp),
        contentAlignment = Alignment.Center,
    ) {
        if (refreshing) {
            CircularProgressIndicator(
                color = Palette.Muted,
                strokeWidth = 1.dp,
                modifier = Modifier.size(16.dp),
            )
        } else {
            Text(text = "Refresh", style = DashboardTypography.labelLarge, color = Palette.Text)
        }
    }
}
