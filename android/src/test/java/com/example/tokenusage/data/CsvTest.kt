package com.example.tokenusage.data

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Instant

/**
 * Pure-JVM tests for CSV escaping and the exact header/row layout of exports.
 */
class CsvTest {

    private val sessionHeader =
        "Session,Last active,Model,Project,Messages,Input,Output,Reasoning,Cache read,Cache write,Fresh,Total"
    private val dailyHeader =
        "Day,Fresh,Input,Output,Reasoning,Cache read,Cache write,Total"

    @Test
    fun escapeCsvCellLeavesPlainTextUntouched() {
        assertEquals("plain", escapeCsvCell("plain"))
        assertEquals("", escapeCsvCell(""))
        assertEquals("Cache read", escapeCsvCell("Cache read"))
    }

    @Test
    fun escapeCsvCellQuotesCommasQuotesAndNewlines() {
        assertEquals("\"a,b\"", escapeCsvCell("a,b"))
        assertEquals("\"he said \"\"hi\"\"\"", escapeCsvCell("he said \"hi\""))
        assertEquals("\"line1\nline2\"", escapeCsvCell("line1\nline2"))
        assertEquals("\"a,b\"\"c\nd\"", escapeCsvCell("a,b\"c\nd"))
    }

    @Test
    fun toCsvEscapesBothHeaderAndRows() {
        val csv = toCsv(
            listOf("Last, active", "Plain"),
            listOf(
                listOf("x", "y\nz"),
                listOf("\"quoted\"", "42"),
            ),
        )
        assertEquals(
            "\"Last, active\",Plain\n" +
                "x,\"y\nz\"\n" +
                "\"\"\"quoted\"\"\",42",
            csv,
        )
    }

    @Test
    fun toCsvWithNoRowsReturnsHeaderOnly() {
        assertEquals("A,B,C", toCsv(listOf("A", "B", "C"), emptyList()))
    }

    @Test
    fun sessionsToCsvUsesExpectedHeaderAndLayout() {
        val updatedAt = Instant.parse("2024-01-01T10:00:00Z").toEpochMilli()
        val session = SessionTotals(
            key = "s1",
            label = "Chat, alpha",
            model = "test/gpt",
            project = "alpha",
            updatedAt = updatedAt,
            messages = 2,
            input = 10,
            output = 5,
            reasoning = 1,
            cacheRead = 100,
            cacheWrite = 2,
            fresh = 16,
            total = 118,
        )

        assertEquals(
            "$sessionHeader\n" +
                "\"Chat, alpha\",2024-01-01T10:00:00.000Z,test/gpt,alpha,2,10,5,1,100,2,16,118",
            sessionsToCsv(listOf(session)),
        )
    }

    @Test
    fun sessionsToCsvDoublesQuotesInLabels() {
        val session = SessionTotals(
            key = "s2",
            label = "He said \"hello\"",
            model = "test/gpt",
            project = "alpha",
            updatedAt = Instant.parse("2024-01-01T10:00:00Z").toEpochMilli(),
            messages = 1,
            input = 0,
            output = 0,
            reasoning = 0,
            cacheRead = 0,
            cacheWrite = 0,
            fresh = 0,
            total = 0,
        )

        assertEquals(
            "$sessionHeader\n" +
                "\"He said \"\"hello\"\"\",2024-01-01T10:00:00.000Z,test/gpt,alpha,1,0,0,0,0,0,0,0",
            sessionsToCsv(listOf(session)),
        )
    }

    @Test
    fun dailyToCsvUsesExpectedHeaderAndLayout() {
        val day = DayTotals(
            day = "2024-01-01",
            input = 10,
            output = 5,
            reasoning = 1,
            cacheRead = 100,
            cacheWrite = 2,
            fresh = 16,
            total = 118,
        )

        assertEquals(
            "$dailyHeader\n2024-01-01,16,10,5,1,100,2,118",
            dailyToCsv(listOf(day)),
        )
    }
}
