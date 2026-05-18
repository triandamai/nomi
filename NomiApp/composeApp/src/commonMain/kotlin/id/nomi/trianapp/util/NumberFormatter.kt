package id.nomi.trianapp.util

import kotlin.math.abs

/**
 * Formats token counts for display.
 * - Under 10,000,000: Formats with thousand separators (e.g., 1.250.000)
 * - 10,000,000 and above: Simplifies to M or B (e.g., 10M, 1.5B)
 */
fun formatTokenCount(count: Long): String {
    val absoluteCount = abs(count)
    return when {
        absoluteCount >= 1_000_000_000 -> {
            val billions = count.toDouble() / 1_000_000_000.0
            if (billions % 1.0 == 0.0) {
                "${billions.toLong()}B"
            } else {
                "${"%.1f".format(billions)}B"
            }
        }
        absoluteCount >= 10_000_000 -> {
            val millions = count.toDouble() / 1_000_000.0
            if (millions % 1.0 == 0.0) {
                "${millions.toLong()}M"
            } else {
                "${"%.1f".format(millions)}M"
            }
        }
        else -> {
            // Pattern #.###.### using '.' as separator
            count.toString()
                .reversed()
                .chunked(3)
                .joinToString(".")
                .reversed()
        }
    }
}
