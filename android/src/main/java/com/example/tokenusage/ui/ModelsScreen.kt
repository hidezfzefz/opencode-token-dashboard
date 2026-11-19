package com.example.tokenusage.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.example.tokenusage.data.UsageSnapshot

@Composable
fun ModelsScreen(
    snapshot: UsageSnapshot,
    modifier: Modifier = Modifier,
) {
    NamedTotalsScreen(
        title = "Models",
        subtitle = "${snapshot.models.size} models and variants across all sessions",
        labelHeader = "Model",
        rows = snapshot.models,
        allTimeTotal = snapshot.allTime.total,
        emptyMessage = "No model data found in the database yet.",
        modifier = modifier,
    )
}
