package com.example.tokenusage.data

import android.content.Context
import android.database.Cursor
import android.database.sqlite.SQLiteDatabase
import java.io.File
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import java.util.TreeMap
import kotlin.math.round

private val SQL = """
    SELECT m.session_id AS sessionId, m.time_created AS timeCreated,
     COALESCE(json_extract(m.data,'$.model.id'),'unknown') AS modelId,
     COALESCE(json_extract(m.data,'$.model.providerID'),'unknown') AS providerId,
     COALESCE(json_extract(m.data,'$.model.variant'),'') AS variant,
     CAST(COALESCE(json_extract(m.data,'$.tokens.input'),0) AS INTEGER) AS input,
     CAST(COALESCE(json_extract(m.data,'$.tokens.output'),0) AS INTEGER) AS output,
     CAST(COALESCE(json_extract(m.data,'$.tokens.reasoning'),0) AS INTEGER) AS reasoning,
     CAST(COALESCE(json_extract(m.data,'$.tokens.cache.read'),0) AS INTEGER) AS cacheRead,
     CAST(COALESCE(json_extract(m.data,'$.tokens.cache.write'),0) AS INTEGER) AS cacheWrite,
     COALESCE(NULLIF(s.title,''), m.session_id) AS sessionTitle,
     COALESCE(p.id,'none') AS projectId,
     COALESCE(NULLIF(p.name,''), NULLIF(p.worktree,''), s.directory, 'Other') AS projectLabel
    FROM session_message m LEFT JOIN session_v2 s ON s.id=m.session_id LEFT JOIN project p ON p.id=s.project_id
    WHERE m.type='assistant' ORDER BY m.time_created ASC
""".trimIndent()

private val DAY_FORMATTER: DateTimeFormatter = DateTimeFormatter.ISO_LOCAL_DATE

/** Day key for an epoch-millisecond timestamp, in the device's local calendar. */
fun dayKey(timeCreatedMillis: Long): String {
    val date = LocalDateTime.ofInstant(Instant.ofEpochMilli(timeCreatedMillis), ZoneId.systemDefault())
        .toLocalDate()
    return DAY_FORMATTER.format(date)
}

private fun dayKey(date: LocalDate): String = DAY_FORMATTER.format(date)

/** Filter sessions by a case-insensitive text query plus exact model/project. */
fun filterSessions(
    rows: List<SessionTotals>,
    query: String,
    model: String,
    project: String,
): List<SessionTotals> {
    val needle = query.trim().lowercase(Locale.ROOT)
    return rows.filter { row ->
        val textMatch = needle.isEmpty() ||
            "${row.label} ${row.model} ${row.project}".lowercase(Locale.ROOT).contains(needle)
        val modelMatch = model.isEmpty() || row.model == model
        val projectMatch = project.isEmpty() || row.project == project
        textMatch && modelMatch && projectMatch
    }
}

private class SessionAcc(
    val key: String,
    val label: String,
    val model: String,
    val project: String,
) {
    var counts: Counts = Counts()
    var messages: Long = 0
    var updatedAt: Long = 0

    fun toTotals(): SessionTotals {
        val t = Totals.from(counts)
        return SessionTotals(
            key = key,
            label = label,
            model = model,
            project = project,
            updatedAt = updatedAt,
            messages = messages,
            input = t.input,
            output = t.output,
            reasoning = t.reasoning,
            cacheRead = t.cacheRead,
            cacheWrite = t.cacheWrite,
            fresh = t.fresh,
            total = t.total,
        )
    }
}

private class NamedAcc(val key: String, val label: String) {
    var counts: Counts = Counts()
    var messages: Long = 0

    fun toTotals(): NamedTotals {
        val t = Totals.from(counts)
        return NamedTotals(
            key = key,
            label = label,
            messages = messages,
            input = t.input,
            output = t.output,
            reasoning = t.reasoning,
            cacheRead = t.cacheRead,
            cacheWrite = t.cacheWrite,
            fresh = t.fresh,
            total = t.total,
        )
    }
}

/**
 * Pure aggregation over already-read rows. No Android/SQLite dependency, so
 * this can be unit-tested on the JVM. [now] supplies the local "today".
 */
fun aggregate(rows: List<UsageRow>, dbPath: String, now: LocalDate): UsageSnapshot {
    val todayKey = dayKey(now)

    var allTime = Counts()
    var today = Counts()
    val days = TreeMap<String, Counts>()
    val sessions = TreeMap<String, SessionAcc>()
    val models = TreeMap<String, NamedAcc>()
    val projects = TreeMap<String, NamedAcc>()

    for (row in rows) {
        allTime += row.counts

        val key = dayKey(row.timeCreated)
        days[key] = (days[key] ?: Counts()) + row.counts
        if (key == todayKey) {
            today += row.counts
        }

        val sessionKey = row.sessionId.ifEmpty { "unknown" }
        val session = sessions.getOrPut(sessionKey) {
            SessionAcc(
                key = sessionKey,
                label = row.sessionTitle.ifEmpty { sessionKey },
                model = row.model,
                project = row.projectLabel,
            )
        }
        session.counts += row.counts
        session.messages += 1
        session.updatedAt = maxOf(session.updatedAt, row.timeCreated)

        val modelKey = row.model
        val model = models.getOrPut(modelKey) { NamedAcc(modelKey, modelKey) }
        model.counts += row.counts
        model.messages += 1

        val projectKey = row.projectId
        val project = projects.getOrPut(projectKey) { NamedAcc(projectKey, row.projectLabel) }
        project.counts += row.counts
        project.messages += 1
    }

    var yesterday = Counts()
    var sevenDays = Counts()
    for (offset in 1..7) {
        val key = dayKey(now.minusDays(offset.toLong()))
        val counts = days[key] ?: continue
        if (offset == 1) {
            yesterday += counts
        }
        sevenDays += counts
    }
    val average = Counts(
        input = round(sevenDays.input / 7.0).toLong(),
        output = round(sevenDays.output / 7.0).toLong(),
        reasoning = round(sevenDays.reasoning / 7.0).toLong(),
        cacheRead = round(sevenDays.cacheRead / 7.0).toLong(),
        cacheWrite = round(sevenDays.cacheWrite / 7.0).toLong(),
    )

    val byTotalDescThenKey = compareByDescending<SessionTotals> { it.total }.thenBy { it.key }
    val namedByTotalDescThenKey = compareByDescending<NamedTotals> { it.total }.thenBy { it.key }

    return UsageSnapshot(
        dbPath = dbPath,
        messageCount = rows.size,
        today = Totals.from(today),
        yesterday = Totals.from(yesterday),
        sevenDayAverage = Totals.from(average),
        allTime = Totals.from(allTime),
        perDay = days.map { (day, counts) ->
            val t = Totals.from(counts)
            DayTotals(
                day = day,
                input = t.input,
                output = t.output,
                reasoning = t.reasoning,
                cacheRead = t.cacheRead,
                cacheWrite = t.cacheWrite,
                fresh = t.fresh,
                total = t.total,
            )
        },
        sessions = sessions.values.map { it.toTotals() }.sortedWith(byTotalDescThenKey),
        models = models.values.map { it.toTotals() }.sortedWith(namedByTotalDescThenKey),
        projects = projects.values.map { it.toTotals() }.sortedWith(namedByTotalDescThenKey),
    )
}

object UsageRepository {
    /** The DB file the user copies into the app's private files directory. */
    fun databaseFile(context: Context): File =
        File(context.filesDir, "token-usage/opencode.db")

    /** Reads the database strictly read-only and returns an aggregated snapshot. */
    fun load(dbFile: File): UsageSnapshot {
        require(dbFile.exists()) {
            "OpenCode database not found at ${dbFile.absolutePath}"
        }
        val db = SQLiteDatabase.openDatabase(dbFile.absolutePath, null, SQLiteDatabase.OPEN_READONLY)
        try {
            val rows = mutableListOf<UsageRow>()
            db.rawQuery(SQL, null).use { cursor ->
                while (cursor.moveToNext()) {
                    rows.add(cursor.toUsageRow())
                }
            }
            return aggregate(rows, dbFile.absolutePath, LocalDate.now())
        } finally {
            db.close()
        }
    }

    // Convenience delegates so callers may use either top-level or object form.
    fun aggregate(rows: List<UsageRow>, dbPath: String, now: LocalDate): UsageSnapshot =
        com.example.tokenusage.data.aggregate(rows, dbPath, now)

    fun filterSessions(
        rows: List<SessionTotals>,
        query: String,
        model: String,
        project: String,
    ): List<SessionTotals> = com.example.tokenusage.data.filterSessions(rows, query, model, project)

    fun dayKey(timeCreatedMillis: Long): String = com.example.tokenusage.data.dayKey(timeCreatedMillis)
}

private fun Cursor.text(name: String): String = getString(getColumnIndexOrThrow(name)) ?: ""
private fun Cursor.number(name: String): Long = getLong(getColumnIndexOrThrow(name))

private fun Cursor.toUsageRow(): UsageRow = UsageRow(
    sessionId = text("sessionId"),
    timeCreated = number("timeCreated"),
    modelId = text("modelId"),
    providerId = text("providerId"),
    variant = text("variant"),
    input = number("input"),
    output = number("output"),
    reasoning = number("reasoning"),
    cacheRead = number("cacheRead"),
    cacheWrite = number("cacheWrite"),
    sessionTitle = text("sessionTitle"),
    projectId = text("projectId"),
    projectLabel = text("projectLabel"),
)
