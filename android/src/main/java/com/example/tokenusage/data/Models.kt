package com.example.tokenusage.data

/**
 * Raw token counts for a single message or an accumulated bucket.
 */
data class Counts(
    val input: Long = 0,
    val output: Long = 0,
    val reasoning: Long = 0,
    val cacheRead: Long = 0,
    val cacheWrite: Long = 0,
) {
    operator fun plus(other: Counts): Counts = Counts(
        input = input + other.input,
        output = output + other.output,
        reasoning = reasoning + other.reasoning,
        cacheRead = cacheRead + other.cacheRead,
        cacheWrite = cacheWrite + other.cacheWrite,
    )
}

/**
 * Counts enriched with the derived [fresh] (non-cached) and [total] figures.
 */
data class Totals(
    val input: Long = 0,
    val output: Long = 0,
    val reasoning: Long = 0,
    val cacheRead: Long = 0,
    val cacheWrite: Long = 0,
    val fresh: Long = 0,
    val total: Long = 0,
) {
    companion object {
        fun from(counts: Counts): Totals = Totals(
            input = counts.input,
            output = counts.output,
            reasoning = counts.reasoning,
            cacheRead = counts.cacheRead,
            cacheWrite = counts.cacheWrite,
            fresh = counts.input + counts.output + counts.reasoning,
            total = counts.input + counts.output + counts.reasoning + counts.cacheRead + counts.cacheWrite,
        )
    }
}

data class DayTotals(
    val day: String,
    val input: Long,
    val output: Long,
    val reasoning: Long,
    val cacheRead: Long,
    val cacheWrite: Long,
    val fresh: Long,
    val total: Long,
)

data class NamedTotals(
    val key: String,
    val label: String,
    val messages: Long,
    val input: Long,
    val output: Long,
    val reasoning: Long,
    val cacheRead: Long,
    val cacheWrite: Long,
    val fresh: Long,
    val total: Long,
)

data class SessionTotals(
    val key: String,
    val label: String,
    val model: String,
    val project: String,
    val updatedAt: Long,
    val messages: Long,
    val input: Long,
    val output: Long,
    val reasoning: Long,
    val cacheRead: Long,
    val cacheWrite: Long,
    val fresh: Long,
    val total: Long,
)

data class UsageSnapshot(
    val dbPath: String,
    val messageCount: Int,
    val today: Totals,
    val yesterday: Totals,
    val sevenDayAverage: Totals,
    val allTime: Totals,
    val perDay: List<DayTotals>,
    val sessions: List<SessionTotals>,
    val models: List<NamedTotals>,
    val projects: List<NamedTotals>,
)

/**
 * One assistant message as returned by the usage query. The raw identifier
 * fields are kept so aggregation happens entirely on the JVM (no SQLite
 * dependency), which keeps it unit-testable.
 */
data class UsageRow(
    val sessionId: String,
    val timeCreated: Long,
    val modelId: String,
    val providerId: String,
    val variant: String,
    val input: Long,
    val output: Long,
    val reasoning: Long,
    val cacheRead: Long,
    val cacheWrite: Long,
    val sessionTitle: String,
    val projectId: String,
    val projectLabel: String,
    val model: String = if (variant.isEmpty()) {
        "$providerId/$modelId"
    } else {
        "$providerId/$modelId#$variant"
    },
) {
    val counts: Counts
        get() = Counts(input, output, reasoning, cacheRead, cacheWrite)
}
