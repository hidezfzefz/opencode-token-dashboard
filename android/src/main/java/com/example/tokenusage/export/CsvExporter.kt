package com.example.tokenusage.export

import android.content.Context
import com.example.tokenusage.data.DayTotals
import com.example.tokenusage.data.SessionTotals
import com.example.tokenusage.data.dailyToCsv
import com.example.tokenusage.data.sessionsToCsv
import java.io.File
import java.io.IOException

/** Outcome of an export, safe to turn directly into a Toast/Snackbar message. */
sealed interface CsvExportResult {
    data class Success(val file: File) : CsvExportResult
    data class Failure(val message: String) : CsvExportResult
}

/**
 * Writes CSV files into the app's own external files directory, so no storage
 * permission or Storage Access Framework picker is required. The files live at
 *
 * `/storage/emulated/0/Android/data/com.example.tokenusage/files/exports/`
 *
 * and are removed automatically when the app is uninstalled.
 */
class CsvExporter(private val context: Context) {

    companion object {
        const val EXPORTS_DIR: String = "exports"
        const val SESSIONS_FILE_NAME: String = "opencode-token-usage-sessions.csv"
        const val DAILY_FILE_NAME: String = "opencode-token-usage-daily.csv"

        /** Short user-facing summary of a result, for Toasts or status text. */
        fun message(result: CsvExportResult): String = when (result) {
            is CsvExportResult.Success ->
                "Saved ${result.file.name} to ${result.file.parentFile?.absolutePath ?: result.file.absolutePath}"
            is CsvExportResult.Failure -> result.message
        }
    }

    /** Directory used for exports, or null when external storage is unavailable. */
    fun exportDirectory(): File? = context.getExternalFilesDir(EXPORTS_DIR)

    fun exportSessions(rows: List<SessionTotals>): CsvExportResult =
        write(SESSIONS_FILE_NAME, sessionsToCsv(rows))

    fun exportDaily(rows: List<DayTotals>): CsvExportResult =
        write(DAILY_FILE_NAME, dailyToCsv(rows))

    // Aliases kept for callers that prefer "write" terminology.
    fun writeSessions(rows: List<SessionTotals>): CsvExportResult = exportSessions(rows)
    fun writeDaily(rows: List<DayTotals>): CsvExportResult = exportDaily(rows)

    private fun write(fileName: String, content: String): CsvExportResult = try {
        val directory = context.getExternalFilesDir(EXPORTS_DIR)
            ?: return CsvExportResult.Failure("External storage is not available on this device.")
        if (!directory.exists() && !directory.mkdirs()) {
            return CsvExportResult.Failure("Could not create export folder: ${directory.absolutePath}")
        }
        val file = File(directory, fileName)
        file.writeText(content, Charsets.UTF_8)
        CsvExportResult.Success(file)
    } catch (error: IOException) {
        CsvExportResult.Failure("Could not write $fileName: ${error.message ?: "I/O error"}")
    } catch (error: SecurityException) {
        CsvExportResult.Failure("Permission denied writing $fileName: ${error.message ?: "security error"}")
    } catch (error: RuntimeException) {
        CsvExportResult.Failure("Could not write $fileName: ${error.message ?: "unexpected error"}")
    }
}
