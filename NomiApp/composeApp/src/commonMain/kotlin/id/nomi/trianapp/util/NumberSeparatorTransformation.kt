package id.nomi.trianapp.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation

class NumberSeparatorTransformation(private val separator: Char = '.') : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val originalText = text.text
        if (originalText.isEmpty()) return TransformedText(text, OffsetMapping.Identity)

        val formattedText = originalText
            .reversed()
            .chunked(3)
            .joinToString(separator.toString())
            .reversed()

        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                if (offset <= 0) return 0
                val realOffset = offset.coerceAtMost(originalText.length)
                
                // Calculate how many separators are added before the current offset position
                // We count from the right side because that's where chunking starts
                val digitsFromRight = originalText.length - realOffset
                val separatorsBeforeFromRight = digitsFromRight / 3
                val totalSeparators = (originalText.length - 1) / 3
                
                return realOffset + (totalSeparators - separatorsBeforeFromRight)
            }

            override fun transformedToOriginal(offset: Int): Int {
                if (offset <= 0) return 0
                val realOffset = offset.coerceAtMost(formattedText.length)
                
                // Count how many non-separator characters are present up to realOffset
                return formattedText.substring(0, realOffset).count { it != separator }
            }
        }

        return TransformedText(AnnotatedString(formattedText), offsetMapping)
    }
}
