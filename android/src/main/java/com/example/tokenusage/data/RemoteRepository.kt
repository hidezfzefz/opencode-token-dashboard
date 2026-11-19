package com.example.tokenusage.data

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.io.BufferedReader
import java.io.InputStreamReader

/**
 * Reads a usage snapshot from the desktop app's local web server over the LAN.
 *
 * Android cannot read another app's private database, so the phone talks to
 * the Rust dashboard running next to OpenCode, e.g. `http://192.168.0.10:3000`.
 * The endpoint is read-only. Both snake_case and camelCase keys are accepted so
 * the client keeps working if the server changes its JSON naming.
 */
class RemoteRepository(private val baseUrl: String) {

    private val normalized: String = baseUrl.trim().trimEnd('/').let {
        if (it.startsWith("http://") || it.startsWith("https://")) it else "http://$it"
    }

    fun load(): UsageSnapshot {
        val connection = URL("$normalized/api/usage").openConnection() as HttpURLConnection
        connection.apply {
            requestMethod = "GET"
            connectTimeout = 5_000
            readTimeout = 10_000
            setRequestProperty("Accept", "application/json")
        }
        try {
            val code = connection.responseCode
            if (code != HttpURLConnection.HTTP_OK) {
                val message = readBody(connection.errorStream).take(200)
                throw IllegalStateException("Server returned HTTP $code: $message")
            }
            return parse(JSONObject(readBody(connection.inputStream)))
        } finally {
            connection.disconnect()
        }
    }

    private fun readBody(stream: java.io.InputStream?): String {
        if (stream == null) return ""
        return BufferedReader(InputStreamReader(stream, Charsets.UTF_8)).use { it.readText() }
    }

    private fun JSONObject.stringOf(vararg keys: String): String {
        keys.forEach { if (has(it)) return optString(it) }
        return ""
    }

    private fun JSONObject.longOf(vararg keys: String): Long {
        keys.forEach { if (has(it)) return optLong(it) }
        return 0L
    }

    private fun JSONObject.intOf(vararg keys: String): Int {
        keys.forEach { if (has(it)) return optInt(it) }
        return 0
    }

    private fun totals(json: JSONObject) = Totals(
        input = json.longOf("input"),
        output = json.longOf("output"),
        reasoning = json.longOf("reasoning"),
        cacheRead = json.longOf("cacheRead", "cache_read"),
        cacheWrite = json.longOf("cacheWrite", "cache_write"),
        fresh = json.longOf("fresh"),
        total = json.longOf("total"),
    )

    private fun day(json: JSONObject) = DayTotals(
        day = json.stringOf("day"),
        input = json.longOf("input"),
        output = json.longOf("output"),
        reasoning = json.longOf("reasoning"),
        cacheRead = json.longOf("cacheRead", "cache_read"),
        cacheWrite = json.longOf("cacheWrite", "cache_write"),
        fresh = json.longOf("fresh"),
        total = json.longOf("total"),
    )

    private fun named(json: JSONObject) = NamedTotals(
        key = json.stringOf("key"),
        label = json.stringOf("label"),
        messages = json.longOf("messages"),
        input = json.longOf("input"),
        output = json.longOf("output"),
        reasoning = json.longOf("reasoning"),
        cacheRead = json.longOf("cacheRead", "cache_read"),
        cacheWrite = json.longOf("cacheWrite", "cache_write"),
        fresh = json.longOf("fresh"),
        total = json.longOf("total"),
    )

    private fun session(json: JSONObject) = SessionTotals(
        key = json.stringOf("key"),
        label = json.stringOf("label"),
        model = json.stringOf("model"),
        project = json.stringOf("project"),
        updatedAt = json.longOf("updatedAt", "updated_at"),
        messages = json.longOf("messages"),
        input = json.longOf("input"),
        output = json.longOf("output"),
        reasoning = json.longOf("reasoning"),
        cacheRead = json.longOf("cacheRead", "cache_read"),
        cacheWrite = json.longOf("cacheWrite", "cache_write"),
        fresh = json.longOf("fresh"),
        total = json.longOf("total"),
    )

    private fun array(json: JSONObject, vararg keys: String): JSONArray {
        keys.forEach { json.optJSONArray(it)?.let { return it } }
        return JSONArray()
    }

    private fun parse(json: JSONObject): UsageSnapshot = UsageSnapshot(
        dbPath = json.stringOf("dbPath", "db_path"),
        messageCount = json.intOf("messageCount", "message_count"),
        today = totals(json.getJSONObject("today")),
        yesterday = totals(json.getJSONObject("yesterday")),
        sevenDayAverage = totals(
            json.optJSONObject("sevenDayAverage") ?: json.getJSONObject("seven_day_average")
        ),
        allTime = totals(json.getJSONObject("allTime") ?: json.getJSONObject("all_time")),
        perDay = array(json, "perDay", "per_day").map(::day),
        sessions = array(json, "sessions").map(::session),
        models = array(json, "models").map(::named),
        projects = array(json, "projects").map(::named),
    )

    private fun <T> JSONArray.map(transform: (JSONObject) -> T): List<T> =
        (0 until length()).map { transform(getJSONObject(it)) }
}
