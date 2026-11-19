package com.example.tokenusage.data

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Pure-JVM tests for the display helpers. No Android framework is involved.
 */
class FormatTest {

    @Test
    fun formatNumberGroupsThousands() {
        assertEquals("0", formatNumber(0L))
        assertEquals("842", formatNumber(842L))
        assertEquals("999", formatNumber(999L))
        assertEquals("1,000", formatNumber(1_000L))
        assertEquals("12,345", formatNumber(12_345L))
        assertEquals("1,234,567", formatNumber(1_234_567L))
    }

    @Test
    fun formatNumberHandlesNegativesAndLongBoundaries() {
        assertEquals("-999", formatNumber(-999L))
        assertEquals("-1,234", formatNumber(-1_234L))
        assertEquals("-1,234,567", formatNumber(-1_234_567L))
        assertEquals("-9,223,372,036,854,775,808", formatNumber(Long.MIN_VALUE))
        assertEquals("9,223,372,036,854,775,807", formatNumber(Long.MAX_VALUE))
    }

    @Test
    fun formatCompactUsesIntlStyleUnits() {
        assertEquals("1.7B", formatCompact(1_714_808_757L))
        assertEquals("34.8M", formatCompact(34_800_000L))
        assertEquals("842", formatCompact(842L))
    }

    @Test
    fun formatCompactCoversUnitBoundariesAndNegatives() {
        assertEquals("0", formatCompact(0L))
        assertEquals("999", formatCompact(999L))
        assertEquals("1K", formatCompact(1_000L))
        assertEquals("1.5K", formatCompact(1_500L))
        assertEquals("1B", formatCompact(1_000_000_000L))
        assertEquals("-1.2K", formatCompact(-1_234L))
        assertEquals("-2.5M", formatCompact(-2_500_000L))
    }

    @Test
    fun formatPercentRoundsAndHandlesZeroWhole() {
        assertEquals("33%", formatPercent(1, 3))
        assertEquals("67%", formatPercent(2, 3))
        assertEquals("50%", formatPercent(1, 2))
        assertEquals("100%", formatPercent(3, 3))
        assertEquals("0%", formatPercent(0, 10))
        assertEquals("0%", formatPercent(5, 0))
        assertEquals("0%", formatPercent(5, -3))
    }
}
