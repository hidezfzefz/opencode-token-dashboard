package com.example.tokenusage.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.example.tokenusage.data.UsageSnapshot

@Composable
fun ProjectsScreen(
    snapshot: UsageSnapshot,
    modifier: Modifier = Modifier,
) {
    NamedTotalsScreen(
        title = "Projects",
        subtitle = "${snapshot.projects.size} projects with assistant messages",
        labelHeader = "Project",
        rows = snapshot.projects,
        allTimeTotal = snapshot.allTime.total,
        emptyMessage = "No project data found in the database yet.",
        modifier = modifier,
    )
}
