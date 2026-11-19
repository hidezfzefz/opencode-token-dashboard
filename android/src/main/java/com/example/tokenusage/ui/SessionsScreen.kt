package com.example.tokenusage.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.example.tokenusage.data.UsageSnapshot
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

private data class ColumnDef(
    val key: String,
    val title: String,
    val weight: Float,
    val align: TextAlign = TextAlign.End,
)

private val SESSION_COLUMNS = listOf(
    ColumnDef("label", "Session", 2f, TextAlign.Start),
    ColumnDef("updatedAt", "Last active", 1f),
    ColumnDef("input", "Input", 1f),
    ColumnDef("output", "Output", 1f),
    ColumnDef("reasoning", "Reasoning", 1f),
    ColumnDef("cacheRead", "Cache read", 1f),
    ColumnDef("fresh", "Fresh", 1f),
    ColumnDef("total", "Total", 1f),
)

@Composable
fun SessionsScreen(
    snapshot: UsageSnapshot,
    query: String,
    onQueryChange: (String) -> Unit,
    onExportSessions: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var modelFilter by remember { mutableStateOf("") }
    var projectFilter by remember { mutableStateOf("") }
    var sortKey by remember { mutableStateOf("total") }
    var ascending by remember { mutableStateOf(false) }
    var modelExpanded by remember { mutableStateOf(false) }
    var projectExpanded by remember { mutableStateOf(false) }

    val modelOptions = remember(snapshot) {
        snapshot.sessions.map { it.model }.distinct().sorted()
    }
    val projectOptions = remember(snapshot) {
        snapshot.sessions.map { it.project }.distinct().sorted()
    }

    val dateFormat = remember { SimpleDateFormat("yyyy-MM-dd", Locale.getDefault()) }
    val rows = remember(snapshot, query, modelFilter, projectFilter, sortKey, ascending) {
        val filtered = Ui.filteredSessions(snapshot.sessions, query, modelFilter, projectFilter)
        filtered.sortedWith { a, b ->
            val comparison = when (sortKey) {
                "label" -> a.label.compareTo(b.label, ignoreCase = true)
                "updatedAt" -> a.updatedAt.compareTo(b.updatedAt)
                "input" -> a.input.compareTo(b.input)
                "output" -> a.output.compareTo(b.output)
                "reasoning" -> a.reasoning.compareTo(b.reasoning)
                "cacheRead" -> a.cacheRead.compareTo(b.cacheRead)
                "fresh" -> a.fresh.compareTo(b.fresh)
                "total" -> a.total.compareTo(b.total)
                else -> 0
            }
            if (ascending) comparison else -comparison
        }
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            PageHeader(
                title = "Sessions",
                subtitle = "${Ui.number(snapshot.sessions.size.toLong())} sessions with assistant messages",
                action = { SmallActionButton("Export CSV", onExportSessions, enabled = rows.isNotEmpty()) },
            )
        }

        item {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = query,
                    onValueChange = onQueryChange,
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    placeholder = { Text("Search sessions, models or projects") },
                    textStyle = DashboardTypography.bodyMedium,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterDropdown(
                        label = "Model",
                        options = modelOptions,
                        value = modelFilter,
                        onValueChange = { modelFilter = it },
                        expanded = modelExpanded,
                        onExpandedChange = { modelExpanded = it },
                        modifier = Modifier.weight(1f),
                    )
                    FilterDropdown(
                        label = "Project",
                        options = projectOptions,
                        value = projectFilter,
                        onValueChange = { projectFilter = it },
                        expanded = projectExpanded,
                        onExpandedChange = { projectExpanded = it },
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        item {
            CardSurface {
                if (rows.isEmpty()) {
                    EmptyText(
                        if (snapshot.sessions.isEmpty()) {
                            "No sessions found in the database yet."
                        } else {
                            "No sessions match this search. Try a different term."
                        },
                    )
                } else {
                    TableHeadRow {
                        SESSION_COLUMNS.forEach { column ->
                            val indicator = if (sortKey == column.key) {
                                if (ascending) "↑" else "↓"
                            } else {
                                ""
                            }
                            HeadCell(
                                text = column.title,
                                weight = column.weight,
                                align = column.align,
                                indicator = indicator,
                                onClick = {
                                    if (sortKey == column.key) {
                                        ascending = !ascending
                                    } else {
                                        sortKey = column.key
                                        ascending = false
                                    }
                                },
                            )
                        }
                    }
                    rows.forEach { session ->
                        TableDataRow {
                            DataCell(
                                session.label,
                                SESSION_COLUMNS[0].weight,
                                TextAlign.Start,
                                numeric = false,
                                color = Palette.Text,
                            )
                            DataCell(
                                dateFormat.format(Date(session.updatedAt)),
                                SESSION_COLUMNS[1].weight,
                            )
                            DataCell(Ui.compact(session.input), SESSION_COLUMNS[2].weight)
                            DataCell(Ui.compact(session.output), SESSION_COLUMNS[3].weight)
                            DataCell(Ui.compact(session.reasoning), SESSION_COLUMNS[4].weight)
                            DataCell(Ui.compact(session.cacheRead), SESSION_COLUMNS[5].weight)
                            DataCell(Ui.compact(session.fresh), SESSION_COLUMNS[6].weight)
                            DataCell(Ui.number(session.total), SESSION_COLUMNS[7].weight)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun FilterDropdown(
    label: String,
    options: List<String>,
    value: String,
    onValueChange: (String) -> Unit,
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(modifier = modifier) {
        val shape = RoundedCornerShape(999.dp)
        TextButton(
            onClick = { onExpandedChange(true) },
            modifier = Modifier
                .fillMaxWidth()
                .border(1.dp, Palette.Hairline, shape)
                .background(Palette.Raised, shape),
        ) {
            Text(
                text = if (value.isEmpty()) "$label: All" else "$label: $value",
                style = DashboardTypography.labelMedium,
                color = Palette.Text,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { onExpandedChange(false) },
            modifier = Modifier.background(Palette.CardAlt),
        ) {
            DropdownMenuItem(
                text = { Text("All", style = DashboardTypography.bodyMedium) },
                onClick = {
                    onValueChange("")
                    onExpandedChange(false)
                },
            )
            options.forEach { option ->
                DropdownMenuItem(
                    text = {
                        Text(option, style = DashboardTypography.bodyMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    },
                    onClick = {
                        onValueChange(option)
                        onExpandedChange(false)
                    },
                )
            }
        }
    }
}
