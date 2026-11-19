package com.example.tokenusage.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.example.tokenusage.data.DayTotals
import com.example.tokenusage.data.UsageSnapshot
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.max
import kotlin.math.pow
import kotlin.math.roundToInt

@Composable
fun OverviewScreen(
    snapshot: UsageSnapshot,
    onOpenSession: (String) -> Unit,
    onExportDaily: () -> Unit,
    modifier: Modifier = Modifier,
    initialAllDays: Boolean = false,
) {
    var showAll by remember { mutableStateOf(initialAllDays) }
    val days = if (showAll) snapshot.perDay else snapshot.perDay.takeLast(30)
    val dailyRows = remember(days) { days.asReversed() }
    val topSessions = remember(snapshot) { snapshot.sessions.take(8) }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(
            horizontal = 16.dp,
            vertical = 24.dp,
        ),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item { PageHeader("Token usage", "Read from ${snapshot.dbPath}") }

        item { StatGrid(snapshot) }

        item {
            CardSurface {
                SectionHeader(
                    title = "Daily usage",
                    action = { RangeToggle(showAll) { showAll = it } },
                )
                Spacer(modifier = Modifier.height(12.dp))
                if (days.isEmpty()) {
                    EmptyText("No assistant messages found in the database yet.")
                } else {
                    UsageChart(days = days, modifier = Modifier.fillMaxWidth().height(260.dp))
                }
                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    text = "Previous 7 days daily average: ${Ui.compact(snapshot.sevenDayAverage.total)} tokens. " +
                        "Yesterday: ${Ui.compact(snapshot.yesterday.total)} tokens. " +
                        "Fresh means input, output and reasoning.",
                    style = DashboardTypography.bodySmall,
                    color = Palette.Muted,
                )
            }
        }

        item {
            CardSurface {
                SectionHeader(
                    title = "Daily detail",
                    action = {
                        SmallActionButton(
                            label = "Export daily CSV",
                            onClick = onExportDaily,
                            enabled = dailyRows.isNotEmpty(),
                        )
                    },
                )
                Spacer(modifier = Modifier.height(12.dp))
                if (dailyRows.isEmpty()) {
                    EmptyText("No assistant messages found in the database yet.")
                } else {
                    TableHeadRow {
                        HeadCell("Day", 1.4f, TextAlign.Start)
                        HeadCell("Fresh", 1f)
                        HeadCell("Cache read", 1f)
                        HeadCell("Total", 1f)
                    }
                    dailyRows.forEach { day ->
                        TableDataRow {
                            DataCell(day.day, 1.4f, TextAlign.Start, numeric = false, color = Palette.Text)
                            DataCell(Ui.compact(day.fresh), 1f)
                            DataCell(Ui.compact(day.cacheRead), 1f)
                            DataCell(Ui.number(day.total), 1f)
                        }
                    }
                }
            }
        }

        item {
            CardSurface {
                SectionHeader(
                    title = "Sessions using the most tokens",
                    action = { SmallTextButton("All sessions") { onOpenSession("") } },
                )
                Spacer(modifier = Modifier.height(12.dp))
                if (topSessions.isEmpty()) {
                    EmptyText("No sessions yet.")
                } else {
                    TableHeadRow {
                        HeadCell("Session", 2f, TextAlign.Start)
                        HeadCell("Model", 2f, TextAlign.Start)
                        HeadCell("Fresh", 1f)
                        HeadCell("Cache read", 1f)
                        HeadCell("Total", 1f)
                    }
                    topSessions.forEach { session ->
                        Box(modifier = Modifier.clickable { onOpenSession(session.label) }) {
                            TableDataRow {
                                DataCell(
                                    session.label,
                                    2f,
                                    TextAlign.Start,
                                    numeric = false,
                                    color = Palette.Text,
                                )
                                DataCell(
                                    session.model,
                                    2f,
                                    TextAlign.Start,
                                    numeric = false,
                                    color = Palette.Muted,
                                )
                                DataCell(Ui.compact(session.fresh), 1f)
                                DataCell(Ui.compact(session.cacheRead), 1f)
                                DataCell(Ui.number(session.total), 1f)
                            }
                        }
                    }
                }
            }
        }
    }
}

private data class Stat(val label: String, val value: String, val meta: String)

@Composable
private fun StatGrid(snapshot: UsageSnapshot) {
    val stats = listOf(
        Stat(
            "Today, all tokens",
            Ui.compact(snapshot.today.total),
            "Yesterday ${Ui.compact(snapshot.yesterday.total)}",
        ),
        Stat(
            "Today, fresh",
            Ui.compact(snapshot.today.fresh),
            "${Ui.percent(snapshot.today.fresh, snapshot.today.total.coerceAtLeast(1L))} of today",
        ),
        Stat(
            "All time, all tokens",
            Ui.compact(snapshot.allTime.total),
            "${Ui.number(snapshot.messageCount.toLong())} assistant messages",
        ),
        Stat(
            "All time, fresh",
            Ui.compact(snapshot.allTime.fresh),
            "${Ui.percent(snapshot.allTime.fresh, snapshot.allTime.total.coerceAtLeast(1L))} of all time",
        ),
    )

    BoxWithConstraints {
        if (maxWidth < 600.dp) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                stats.chunked(2).forEach { pair ->
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        pair.forEach { Box(modifier = Modifier.weight(1f)) { StatCard(it) } }
                    }
                }
            }
        } else {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                stats.forEach { Box(modifier = Modifier.weight(1f)) { StatCard(it) } }
            }
        }
    }
}

@Composable
private fun StatCard(stat: Stat) {
    CardSurface(modifier = Modifier.fillMaxWidth()) {
        Column {
            Text(stat.label, style = DashboardTypography.bodySmall, color = Palette.Muted)
            Spacer(modifier = Modifier.height(8.dp))
            Text(stat.value, style = StatValueStyle, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Spacer(modifier = Modifier.height(8.dp))
            Text(stat.meta, style = DashboardTypography.bodySmall, color = Palette.Muted)
        }
    }
}

@Composable
private fun RangeToggle(allTime: Boolean, onChange: (Boolean) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        RangePill("Last 30 days", active = !allTime) { onChange(false) }
        RangePill("All time", active = allTime) { onChange(true) }
    }
}

@Composable
private fun RangePill(label: String, active: Boolean, onClick: () -> Unit) {
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = Modifier
            .background(if (active) Palette.Raised else Palette.Card, shape)
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = DashboardTypography.labelMedium,
            color = if (active) Palette.Text else Palette.Muted,
        )
    }
}

/**
 * Stacked area chart drawn directly with Compose Canvas: horizontal grid,
 * compact axis labels, a fresh area and a cache-read area stacked on top,
 * plus a press/hover guide and tooltip with exact figures.
 */
@Composable
private fun UsageChart(days: List<DayTotals>, modifier: Modifier = Modifier) {
    var selectedIndex by remember(days) { mutableStateOf<Int?>(null) }
    val points = remember(days) {
        days.map { it.fresh to (it.cacheRead + it.cacheWrite) }
    }
    val niceMax = remember(points) {
        niceCeiling((points.maxOfOrNull { it.first + it.second } ?: 0L).coerceAtLeast(1L))
    }

    BoxWithConstraints(modifier = modifier) {
        val widthPx = constraints.maxWidth.toFloat()
        val labelZone = with(LocalDensity.current) { 22.dp.toPx() }
        val plotBottom = constraints.maxHeight.toFloat() - labelZone
        val gridCount = 4

        fun xAt(index: Int): Float =
            if (points.size <= 1) widthPx / 2f else index * (widthPx / (points.size - 1).toFloat())

        fun yAt(value: Long): Float =
            plotBottom - (value.toFloat() / niceMax.toFloat()) * plotBottom

        val active = selectedIndex?.coerceIn(points.indices)

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(points.size) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            val change = event.changes.firstOrNull() ?: continue
                            when (event.type) {
                                PointerEventType.Press, PointerEventType.Move -> {
                                    val ratio = (change.position.x / widthPx).coerceIn(0f, 1f)
                                    selectedIndex = (ratio * (points.size - 1)).roundToInt()
                                        .coerceIn(0, points.size - 1)
                                }

                                PointerEventType.Release -> selectedIndex = null
                                else -> Unit
                            }
                        }
                    }
                },
        ) {
            for (i in 0..gridCount) {
                val y = plotBottom - (i.toFloat() / gridCount) * plotBottom
                drawLine(Palette.Hairline, Offset(0f, y), Offset(widthPx, y), strokeWidth = 1f)
            }

            fun areaPath(top: (Int) -> Long, bottom: (Int) -> Long): Path {
                val path = Path()
                path.moveTo(xAt(0), yAt(bottom(0)))
                for (i in points.indices) path.lineTo(xAt(i), yAt(top(i)))
                for (i in points.indices.reversed()) path.lineTo(xAt(i), yAt(bottom(i)))
                path.close()
                return path
            }

            drawPath(
                path = areaPath({ points[it].first + points[it].second }, { points[it].first }),
                color = Palette.Raised.copy(alpha = 0.95f),
            )
            drawPath(
                path = areaPath({ points[it].first }, { 0L }),
                color = Palette.Muted.copy(alpha = 0.4f),
            )

            val stroke = 1.5.dp.toPx()
            for (i in 0 until points.size - 1) {
                drawLine(
                    color = Palette.Secondary,
                    start = Offset(xAt(i), yAt(points[i].first)),
                    end = Offset(xAt(i + 1), yAt(points[i + 1].first)),
                    strokeWidth = stroke,
                )
                drawLine(
                    color = Palette.Faint,
                    start = Offset(xAt(i), yAt(points[i].first + points[i].second)),
                    end = Offset(xAt(i + 1), yAt(points[i + 1].first + points[i + 1].second)),
                    strokeWidth = 1.dp.toPx(),
                )
            }

            if (active != null) {
                val x = xAt(active)
                drawLine(Palette.HairlineStrong, Offset(x, 0f), Offset(x, plotBottom), strokeWidth = 1f)
                drawCircle(Palette.Text, radius = 3.dp.toPx(), center = Offset(x, yAt(points[active].first)))
            }

        }

        // Axis labels are regular composables; DrawScope no longer exposes a native canvas.
        for (i in 0..gridCount) {
            val value = niceMax * i / gridCount
            val y = plotBottom - (i.toFloat() / gridCount) * plotBottom
            Text(
                text = Ui.compact(value),
                style = NumericLabelStyle,
                color = Palette.Muted,
                modifier = Modifier.offset {
                    IntOffset(4, (y - 14f).roundToInt())
                },
            )
        }

        val labelIndices = listOf(0, (points.size - 1) / 2, points.size - 1).distinct()
        labelIndices.forEach { index ->
            Text(
                text = days[index].day.takeLast(5),
                style = NumericLabelStyle,
                color = Palette.Muted,
                modifier = Modifier.offset {
                    IntOffset(
                        (xAt(index) - 20f).roundToInt().coerceAtLeast(0),
                        (plotBottom + 4f).roundToInt(),
                    )
                },
            )
        }

        if (active != null) {
            val tipWidth = with(LocalDensity.current) { 160.dp.toPx() }
            val left = (xAt(active) - tipWidth / 2f).coerceIn(0f, (widthPx - tipWidth).coerceAtLeast(0f))
            val day = days[active]
            val fresh = points[active].first
            val cache = points[active].second
            val shape = RoundedCornerShape(12.dp)
            Box(
                modifier = Modifier.offset { IntOffset(left.roundToInt(), 0) },
            ) {
                Column(
                    modifier = Modifier
                        .width(160.dp)
                        .background(Palette.CardAlt.copy(alpha = 0.96f), shape)
                        .border(1.dp, Palette.Hairline, shape)
                        .padding(10.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    Text(day.day, style = DashboardTypography.labelMedium, color = Palette.Text)
                    Text("Fresh  ${Ui.compact(fresh)}", style = DashboardTypography.bodySmall, color = Palette.Muted)
                    Text("Cache read  ${Ui.compact(cache)}", style = DashboardTypography.bodySmall, color = Palette.Muted)
                    Text("Total  ${Ui.number(day.total)}", style = DashboardTypography.bodySmall, color = Palette.Muted)
                }
            }
        }
    }
}

/** Round a value up to a readable axis maximum (1, 2, 2.5, 5, 10 × 10^n). */
private fun niceCeiling(value: Long): Long {
    if (value <= 0) return 1L
    val magnitude = 10.0.pow(floor(log10(value.toDouble())))
    val normalized = value / magnitude
    val nice = when {
        normalized <= 1.0 -> 1.0
        normalized <= 2.0 -> 2.0
        normalized <= 2.5 -> 2.5
        normalized <= 5.0 -> 5.0
        else -> 10.0
    }
    return max(1L, ceil(nice * magnitude).toLong())
}
