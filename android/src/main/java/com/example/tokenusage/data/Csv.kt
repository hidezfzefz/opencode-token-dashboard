package com.example.tokenusage.data

import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

private val SESSION_HEADER = listOf(
    "Session",
    "Last active",
    "Model",
    "Project",
    "Messages",
    "Input",
    "Output",
    "Reasoning",
    "Cache read",
    "Cache write",
    "Fresh",
    "Total",
)

private val DAILY_HEADER = listOf(
    "Day",
    "Fresh",
    "Input",
    "Output",
    "Reasoning",
    "Cache read",
    "Cache write",
    "Total",
)

/** ISO-8601 UTC instant with millisecond precision, e.g. 2024-01-01T10:00:00.000Z. */
private val ISO_MILLIS_UTC: DateTimeFormatter =
    DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'").withZone(ZoneOffset.UTC)

private fun isoUtc(millis: Long): String = try {
    ISO_MILLIS_UTC.format(Instant.ofEpochMilli(millis))
} catch (_: RuntimeException) {
    ""
}

/** Quotes a cell when it contains a comma, quote or newline, doubling quotes. */
fun escapeCsvCell(value: String): String = if (value.any { it == ',' || it == '"' || it == '\n' }) {
    "\"${value.replace("\"", "\"\"")}\""
} else {
    value
}

fun toCsv(header: List<String>, rows: List<List<String>>): String = buildString {
    append(header.joinToString(",") { escapeCsvCell(it) })
    for (row in rows) {
        append('\n')
        append(row.joinToString(",") { escapeCsvCell(it) })
    }
}

fun sessionsToCsv(rows: List<SessionTotals>): String = toCsv(
    SESSION_HEADER,
    rows.map { row ->
        listOf(
            row.label,
            isoUtc(row.updatedAt),
            row.model,
            row.project,
            row.messages.toString(),
            row.input.toString(),
            row.output.toString(),
            row.reasoning.toString(),
            row.cacheRead.toString(),
            row.cacheWrite.toString(),
            row.fresh.toString(),
            row.total.toString(),
        )
    },
)

fun dailyToCsv(rows: List<DayTotals>): String = toCsv(
    DAILY_HEADER,
    rows.map { row ->
        listOf(
            row.day,
            row.fresh.toString(),
            row.input.toString(),
            row.output.toString(),
            row.reasoning.toString(),
            row.cacheRead.toString(),
            row.cacheWrite.toString(),
            row.total.toString(),
        )
    },
)
