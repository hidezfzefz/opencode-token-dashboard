package com.example.tokenusage.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.example.tokenusage.data.NamedTotals
import com.example.tokenusage.data.UsageSnapshot

/**
 * Root composable. All state is passed in or held here so the individual
 * screens stay stateless and the whole thing can be driven by state hoisted
 * from MainActivity (or anywhere else).
 */
@Composable
fun TokenUsageApp(
    snapshot: UsageSnapshot?,
    error: String?,
    onRefresh: () -> Unit,
    onExportSessions: () -> Unit,
    onExportDaily: () -> Unit,
    modifier: Modifier = Modifier,
    refreshing: Boolean = false,
    initialTab: DashboardTab = DashboardTab.Overview,
    initialSessionQuery: String = "",
    autoSyncSeconds: Int = 5,
    allDays: Boolean = false,
    onAutoSyncChange: (Int) -> Unit = {},
    onAllDaysChange: (Boolean) -> Unit = {},
    exportStatus: String? = null,
    remoteUrl: String = "",
    onRemoteUrlChange: (String) -> Unit = {},
) {
    var selectedTab by rememberSaveable { mutableStateOf(initialTab) }
    var sessionQuery by rememberSaveable { mutableStateOf(initialSessionQuery) }

    MaterialTheme(colorScheme = DashboardColorScheme, typography = DashboardTypography) {
        Scaffold(
            modifier = modifier.fillMaxSize().background(Palette.Background),
            containerColor = Palette.Background,
            topBar = {
                TopBar(
                    selectedTab = selectedTab,
                    onTabSelected = { selectedTab = it },
                    onRefresh = onRefresh,
                    refreshing = refreshing,
                    autoSyncSeconds = autoSyncSeconds,
                    onAutoSyncChange = onAutoSyncChange,
                    allDays = allDays,
                    onAllDaysChange = onAllDaysChange,
                    remoteUrl = remoteUrl,
                    onRemoteUrlChange = onRemoteUrlChange,
                )
            },
        ) { innerPadding ->
            val contentModifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)

            when {
                error != null -> ErrorContent(error, onRefresh, contentModifier)
                snapshot == null -> LoadingContent(contentModifier)
                else -> when (selectedTab) {
                    DashboardTab.Overview -> OverviewScreen(
                        snapshot = snapshot,
                        onOpenSession = { label ->
                            sessionQuery = label
                            selectedTab = DashboardTab.Sessions
                        },
                        onExportDaily = onExportDaily,
                        modifier = contentModifier,
                        initialAllDays = allDays,
                    )

                    DashboardTab.Sessions -> SessionsScreen(
                        snapshot = snapshot,
                        query = sessionQuery,
                        onQueryChange = { sessionQuery = it },
                        onExportSessions = onExportSessions,
                        modifier = contentModifier,
                    )

                    DashboardTab.Models -> ModelsScreen(snapshot, contentModifier)
                    DashboardTab.Projects -> ProjectsScreen(snapshot, contentModifier)
                }
            }

            if (exportStatus != null) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.BottomCenter,
                ) {
                    androidx.compose.material3.Surface(
                        color = Palette.Card,
                        shape = RoundedCornerShape(12.dp),
                        shadowElevation = 8.dp,
                        modifier = Modifier.padding(16.dp),
                    ) {
                        Text(
                            exportStatus,
                            color = Palette.Text,
                            style = DashboardTypography.bodyMedium,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun LoadingContent(modifier: Modifier) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Text("Loading usage data…", style = DashboardTypography.bodyMedium, color = Palette.Muted)
    }
}

@Composable
private fun ErrorContent(message: String, onRetry: () -> Unit, modifier: Modifier) {
    Column(
        modifier = modifier.padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("Could not load token usage", style = DashboardTypography.headlineSmall, color = Palette.Text)
        Text(message, style = DashboardTypography.bodyMedium, color = Palette.Muted)
        SmallActionButton(label = "Retry", onClick = onRetry)
    }
}

@Composable
internal fun PageHeader(
    title: String,
    subtitle: String? = null,
    action: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = title, style = DashboardTypography.headlineSmall, color = Palette.Text)
            if (subtitle != null) {
                Text(
                    text = subtitle,
                    style = DashboardTypography.bodySmall,
                    color = Palette.Muted,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        if (action != null) {
            Spacer(modifier = Modifier.width(12.dp))
            action()
        }
    }
}

@Composable
internal fun SectionHeader(
    title: String,
    action: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(text = title, style = DashboardTypography.headlineSmall, color = Palette.Text)
        if (action != null) action()
    }
}

@Composable
internal fun SmallActionButton(
    label: String,
    onClick: () -> Unit,
    enabled: Boolean = true,
) {
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .background(if (enabled) Palette.Raised else Palette.Card, shape)
            .border(1.dp, Palette.Hairline, shape)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 7.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = DashboardTypography.labelMedium,
            color = if (enabled) Palette.Text else Palette.Muted,
        )
    }
}

@Composable
internal fun SmallTextButton(label: String, onClick: () -> Unit) {
    TextButton(onClick = onClick) {
        Text(text = label, style = DashboardTypography.labelMedium, color = Palette.Muted)
    }
}

@Composable
internal fun EmptyText(message: String) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 40.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = message,
            style = DashboardTypography.bodyMedium,
            color = Palette.Muted,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
internal fun TableHeadRow(content: @Composable RowScope.() -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        content()
    }
    Hairline()
}

@Composable
internal fun TableDataRow(content: @Composable RowScope.() -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        content()
    }
    Hairline()
}

@Composable
internal fun RowScope.HeadCell(
    text: String,
    weight: Float = 1f,
    align: TextAlign = TextAlign.Start,
    indicator: String = "",
    onClick: (() -> Unit)? = null,
) {
    val clickModifier = if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier
    Box(
        modifier = Modifier.weight(weight).then(clickModifier),
        contentAlignment = if (align == TextAlign.End) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Text(
            text = if (indicator.isEmpty()) text else "$text $indicator",
            style = DashboardTypography.labelMedium,
            color = Palette.Muted,
            textAlign = align,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun RowScope.DataCell(
    text: String,
    weight: Float = 1f,
    align: TextAlign = TextAlign.End,
    numeric: Boolean = true,
    color: Color = Palette.Secondary,
    style: TextStyle = NumericBodyStyle,
) {
    Text(
        text = text,
        style = if (numeric) style else DashboardTypography.bodyMedium,
        color = color,
        textAlign = align,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.weight(weight),
    )
}

/** Track-and-fill share indicator used by the Models and Projects tables. */
@Composable
internal fun ShareBar(fraction: Float) {
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .size(width = 72.dp, height = 4.dp)
            .background(Palette.Raised, shape),
    ) {
        Box(
            modifier = Modifier
                .fillMaxHeight()
                .fillMaxWidth(fraction.coerceIn(0f, 1f))
                .background(Palette.Muted, shape),
        )
    }
}

/** Shared implementation for the Models and Projects screens. */
@Composable
internal fun NamedTotalsScreen(
    title: String,
    subtitle: String,
    labelHeader: String,
    rows: List<NamedTotals>,
    allTimeTotal: Long,
    emptyMessage: String,
    modifier: Modifier = Modifier,
) {
    val max = rows.firstOrNull()?.total ?: 0L
    val denominator = allTimeTotal.coerceAtLeast(1L)

    androidx.compose.foundation.lazy.LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(
            horizontal = 16.dp,
            vertical = 24.dp,
        ),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item { PageHeader(title, subtitle) }
        item {
            CardSurface {
                if (rows.isEmpty()) {
                    EmptyText(emptyMessage)
                } else {
                    TableHeadRow {
                        HeadCell(labelHeader, 2f, TextAlign.Start)
                        HeadCell("Messages", 1f)
                        HeadCell("Fresh", 1f)
                        HeadCell("Cache read", 1f)
                        HeadCell("Total", 1f)
                        HeadCell("Share", 1f)
                    }
                    rows.forEach { row ->
                        TableDataRow {
                            DataCell(row.label, 2f, TextAlign.Start, numeric = false, color = Palette.Text)
                            DataCell(Ui.number(row.messages), 1f)
                            DataCell(Ui.compact(row.fresh), 1f)
                            DataCell(Ui.compact(row.cacheRead), 1f)
                            DataCell(Ui.number(row.total), 1f)
                            Box(
                                modifier = Modifier.weight(1f),
                                contentAlignment = Alignment.CenterEnd,
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    ShareBar(if (max <= 0) 0f else row.total.toFloat() / max.toFloat())
                                    Spacer(modifier = Modifier.width(8.dp))
                                    Text(
                                        text = Ui.percent(row.total, denominator),
                                        style = NumericLabelStyle,
                                        color = Palette.Muted,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
