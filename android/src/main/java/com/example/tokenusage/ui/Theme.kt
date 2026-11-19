package com.example.tokenusage.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.tokenusage.data.SessionTotals
import com.example.tokenusage.data.filterSessions
import com.example.tokenusage.data.formatCompact
import com.example.tokenusage.data.formatNumber
import com.example.tokenusage.data.formatPercent

/**
 * Visual tokens copied from the web app's stylesheet (static/style.css).
 * These are the only colours the dashboard uses: black canvas, near-black
 * cards, white hairlines and warm white text.
 */
internal object Palette {
    val Background: Color = Color(0xFF000000)
    val Card: Color = Color(0xFF0F0F0F)
    val CardAlt: Color = Color(0xFF171717)
    val Hairline: Color = Color(0x1AFFFFFF) // rgba(255,255,255,.10)
    val HairlineStrong: Color = Color(0x59FFFFFF) // rgba(255,255,255,.35)
    val Text: Color = Color(0xFFFFF3F0)
    val Secondary: Color = Color(0xCCFFFFFF) // 80% white
    val Muted: Color = Color(0xA6FFFFFF) // 65% white
    val Faint: Color = Color(0x40FFFFFF) // 25% white
    val Raised: Color = Color(0x0DFFFFFF) // rgba(255,255,255,.05)
    val Primary: Color = Color(0xFFF4F4F5)
    val OnPrimary: Color = Color(0xFF0A0A0A)
}

internal val DashboardColorScheme: ColorScheme = darkColorScheme(
    primary = Palette.Primary,
    onPrimary = Palette.OnPrimary,
    primaryContainer = Palette.Raised,
    onPrimaryContainer = Palette.Text,
    secondary = Palette.Secondary,
    onSecondary = Palette.OnPrimary,
    surface = Palette.Background,
    onSurface = Palette.Text,
    surfaceVariant = Palette.Card,
    onSurfaceVariant = Palette.Muted,
    background = Palette.Background,
    onBackground = Palette.Text,
    outline = Palette.HairlineStrong,
    outlineVariant = Palette.Hairline,
    error = Color(0xFFFFB4AB),
    onError = Color(0xFF690005),
    scrim = Color(0xFF000000),
)

/** Fixed-width figures, the Compose equivalent of CSS `font-variant-numeric: tabular-nums`. */
private fun tabular(style: TextStyle): TextStyle =
    style.copy(fontFamily = FontFamily.Monospace)

internal val DashboardTypography: Typography = Typography(
    headlineSmall = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 20.sp,
        lineHeight = 28.sp,
        color = Palette.Text,
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 16.sp,
        lineHeight = 24.sp,
        color = Palette.Text,
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        color = Palette.Text,
    ),
    bodySmall = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 12.sp,
        lineHeight = 18.sp,
        color = Palette.Muted,
    ),
    labelLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        color = Palette.Text,
    ),
    labelMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        color = Palette.Muted,
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 11.sp,
        lineHeight = 16.sp,
        color = Palette.Muted,
    ),
)

internal val NumericBodyStyle: TextStyle = tabular(DashboardTypography.bodyMedium)

internal val NumericLabelStyle: TextStyle = tabular(DashboardTypography.labelMedium)

internal val StatValueStyle: TextStyle = TextStyle(
    fontFamily = FontFamily.Monospace,
    fontWeight = FontWeight.Normal,
    fontSize = 30.sp,
    lineHeight = 34.sp,
    color = Palette.Text,
)

/**
 * Single isolation point for the formatting/filtering helpers owned by the
 * `com.example.tokenusage.data` package. Screens only talk to this object, so
 * any helper rename is a one-file change. The helpers mirror the web app's
 * `formatNumber` / `formatCompact` / `formatPercent` / `filterSessions`.
 */
internal object Ui {
    fun number(value: Long): String = formatNumber(value)

    fun compact(value: Long): String = formatCompact(value)

    fun percent(part: Long, whole: Long): String = formatPercent(part, whole)

    fun filteredSessions(
        rows: List<SessionTotals>,
        query: String,
        model: String,
        project: String,
    ): List<SessionTotals> = filterSessions(rows, query, model, project)
}

/** A card: near-black fill, hairline border, generous radius. */
@Composable
internal fun CardSurface(
    modifier: Modifier = Modifier,
    content: @Composable BoxScope.() -> Unit,
) {
    val shape = RoundedCornerShape(16.dp)
    Box(
        modifier = modifier
            .background(Palette.Card, shape)
            .border(1.dp, Palette.Hairline, shape)
            .padding(16.dp),
        content = content,
    )
}

/** One-pixel white hairline used between table rows and under the top bar. */
@Composable
internal fun Hairline(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(1.dp)
            .background(Palette.Hairline),
    )
}
