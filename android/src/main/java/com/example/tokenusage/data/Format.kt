package com.example.tokenusage.data

import java.util.Locale
import kotlin.math.round

/**
 * Groups digits with commas, mirroring Intl / Rust formatting exactly,
 * including negative values and Long.MIN_VALUE (via string handling).
 */
fun formatNumber(value: Long): String {
    val text = value.toString()
    val negative = text.startsWith("-")
    val digits = if (negative) text.substring(1) else text

    val out = StringBuilder(digits.length + digits.length / 3 + 1)
    if (negative) out.append('-')
    for ((index, ch) in digits.withIndex()) {
        if (index > 0 && (digits.length - index) % 3 == 0) {
            out.append(',')
        }
        out.append(ch)
    }
    return out.toString()
}

private fun compactUnit(value: Double, suffix: String): String {
    val rounded = round(value * 10.0) / 10.0
    val text = String.format(Locale.US, "%.1f", rounded)
        .trimEnd('0')
        .trimEnd('.')
    return "$text$suffix"
}

/**
 * Intl-like compact notation, rounded to one decimal with a trailing ".0"
 * removed: 1_714_808_757 -> "1.7B", 34_800_000 -> "34.8M", 842 -> "842".
 */
fun formatCompact(value: Long): String {
    val abs = kotlin.math.abs(value.toDouble())
    return when {
        abs >= 1_000_000_000.0 -> compactUnit(value / 1_000_000_000.0, "B")
        abs >= 1_000_000.0 -> compactUnit(value / 1_000_000.0, "M")
        abs >= 1_000.0 -> compactUnit(value / 1_000.0, "K")
        else -> value.toString()
    }
}

/** Percentage rounded to the nearest whole number, "0%" when [whole] <= 0. */
fun formatPercent(part: Long, whole: Long): String {
    if (whole <= 0) return "0%"
    val percent = round(part.toDouble() / whole.toDouble() * 100.0).toLong()
    return "$percent%"
}
