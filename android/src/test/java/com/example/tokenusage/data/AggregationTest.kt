package com.example.tokenusage.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalDate
import java.time.ZoneId

/**
 * Pure-JVM tests for the aggregation and session filtering logic.
 *
 * The fixture deliberately mirrors the Rust dashboard's own aggregation test:
 * two assistant rows on two consecutive local dates whose fresh tokens sum to
 * 41, cache reads to 300 and total tokens to 341.
 */
class AggregationTest {

    private val now = LocalDate.of(2024, 6, 15)
    private val today = now
    private val yesterday = now.minusDays(1)

    /** Noon avoids DST edge cases while still landing on the intended local date. */
    private fun millisOn(date: LocalDate): Long =
        date.atTime(12, 0).atZone(ZoneId.systemDefault()).toInstant().toEpochMilli()

    private fun row(
        date: LocalDate,
        modelId: String,
        input: Long = 0,
        output: Long = 0,
        reasoning: Long = 0,
        cacheRead: Long = 0,
        cacheWrite: Long = 0,
        sessionId: String = "s1",
        sessionTitle: String = "First session",
        providerId: String = "test",
        variant: String = "",
        projectId: String = "p1",
        projectLabel: String = "alpha",
    ): UsageRow = UsageRow(
        sessionId = sessionId,
        timeCreated = millisOn(date),
        modelId = modelId,
        providerId = providerId,
        variant = variant,
        input = input,
        output = output,
        reasoning = reasoning,
        cacheRead = cacheRead,
        cacheWrite = cacheWrite,
        sessionTitle = sessionTitle,
        projectId = projectId,
        projectLabel = projectLabel,
    )

    private fun fixtureRows() = listOf(
        row(today, modelId = "gpt-test", input = 10, output = 5, cacheRead = 100),
        row(yesterday, modelId = "other-model", input = 20, output = 6, cacheRead = 200),
    )

    private fun session(label: String, model: String, project: String) = SessionTotals(
        key = label,
        label = label,
        model = model,
        project = project,
        updatedAt = 0,
        messages = 1,
        input = 0,
        output = 0,
        reasoning = 0,
        cacheRead = 0,
        cacheWrite = 0,
        fresh = 0,
        total = 0,
    )

    @Test
    fun dayKeyHelperUsesTheLocalCalendar() {
        assertEquals(today.toString(), dayKey(millisOn(today)))
        assertEquals(yesterday.toString(), dayKey(millisOn(yesterday)))
    }

    @Test
    fun aggregatesAllTimeTodayAndYesterday() {
        val snapshot = aggregate(fixtureRows(), "/tmp/opencode.db", now)

        assertEquals("/tmp/opencode.db", snapshot.dbPath)
        assertEquals(2, snapshot.messageCount)

        assertEquals(30L, snapshot.allTime.input)
        assertEquals(11L, snapshot.allTime.output)
        assertEquals(0L, snapshot.allTime.reasoning)
        assertEquals(300L, snapshot.allTime.cacheRead)
        assertEquals(0L, snapshot.allTime.cacheWrite)
        assertEquals(41L, snapshot.allTime.fresh)
        assertEquals(341L, snapshot.allTime.total)

        assertEquals(15L, snapshot.today.fresh)
        assertEquals(115L, snapshot.today.total)
        assertEquals(100L, snapshot.today.cacheRead)

        assertEquals(26L, snapshot.yesterday.fresh)
        assertEquals(226L, snapshot.yesterday.total)
        assertEquals(200L, snapshot.yesterday.cacheRead)
    }

    @Test
    fun sevenDayAverageExcludesTodaysTokens() {
        val snapshot = aggregate(fixtureRows(), "/tmp/opencode.db", now)
        val average = snapshot.sevenDayAverage

        // Only yesterday falls within the 1..=7 day lookback window.
        assertEquals(3L, average.input) // round(20 / 7)
        assertEquals(1L, average.output) // round(6 / 7)
        assertEquals(29L, average.cacheRead) // round(200 / 7)
        assertEquals(0L, average.cacheWrite)
        assertEquals(4L, average.fresh)
        assertEquals(33L, average.total)
    }

    @Test
    fun perDayHasOneEntryPerDistinctLocalDate() {
        val snapshot = aggregate(fixtureRows(), "/tmp/opencode.db", now)

        assertEquals(2, snapshot.perDay.size)
        assertEquals(yesterday.toString(), snapshot.perDay[0].day)
        assertEquals(today.toString(), snapshot.perDay[1].day)
        assertEquals(226L, snapshot.perDay[0].total)
        assertEquals(115L, snapshot.perDay[1].total)
    }

    @Test
    fun groupsSessionsModelsAndProjects() {
        val snapshot = aggregate(fixtureRows(), "/tmp/opencode.db", now)

        assertEquals(1, snapshot.sessions.size)
        val session = snapshot.sessions[0]
        assertEquals("s1", session.key)
        assertEquals("First session", session.label)
        assertEquals(2L, session.messages)
        assertEquals(millisOn(today), session.updatedAt)
        assertEquals(41L, session.fresh)
        assertEquals(341L, session.total)

        assertEquals(2, snapshot.models.size)
        assertEquals("test/other-model", snapshot.models[0].key)
        assertEquals(226L, snapshot.models[0].total)
        assertEquals(1L, snapshot.models[0].messages)
        assertEquals("test/gpt-test", snapshot.models[1].key)
        assertEquals(115L, snapshot.models[1].total)
        assertEquals(1L, snapshot.models[1].messages)

        assertEquals(1, snapshot.projects.size)
        assertEquals("p1", snapshot.projects[0].key)
        assertEquals("alpha", snapshot.projects[0].label)
        assertEquals(2L, snapshot.projects[0].messages)
        assertEquals(341L, snapshot.projects[0].total)
    }

    @Test
    fun emptyInputProducesAnEmptyZeroedSnapshot() {
        val snapshot = aggregate(emptyList(), "none.db", now)

        assertEquals(0, snapshot.messageCount)
        assertEquals(0L, snapshot.allTime.total)
        assertEquals(0L, snapshot.today.total)
        assertEquals(0L, snapshot.yesterday.total)
        assertEquals(0L, snapshot.sevenDayAverage.cacheRead)
        assertTrue(snapshot.perDay.isEmpty())
        assertTrue(snapshot.sessions.isEmpty())
        assertTrue(snapshot.models.isEmpty())
        assertTrue(snapshot.projects.isEmpty())
    }

    @Test
    fun filterSessionsCombinesTextModelAndProject() {
        val rows = listOf(
            session("Alpha chat", "provider/a", "/work/one"),
            session("Beta chat", "provider/b", "/work/two"),
        )

        assertEquals(2, filterSessions(rows, "", "", "").size)
        assertEquals(1, filterSessions(rows, "alpha", "", "").size)
        assertEquals("Alpha chat", filterSessions(rows, "alpha", "", "").first().label)
        assertEquals(1, filterSessions(rows, "  ALPHA  ", "", "").size)
        assertEquals(1, filterSessions(rows, "", "provider/b", "").size)
        assertEquals("Beta chat", filterSessions(rows, "", "provider/b", "").first().label)
        assertEquals(1, filterSessions(rows, "", "", "/work/one").size)
        assertEquals(1, filterSessions(rows, "chat", "provider/a", "/work/one").size)
        assertTrue(filterSessions(rows, "", "provider/a", "/work/two").isEmpty())
        assertTrue(filterSessions(rows, "missing", "", "").isEmpty())
    }
}
