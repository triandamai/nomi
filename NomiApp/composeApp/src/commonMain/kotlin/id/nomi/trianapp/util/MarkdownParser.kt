package id.nomi.trianapp.util

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.graphics.Color

sealed class MarkdownBlock {
    data class Text(val annotatedString: AnnotatedString) : MarkdownBlock()
    data class Code(val rawCode: String, val language: String) : MarkdownBlock()
}

object MarkdownParser {
    fun parse(content: String): List<MarkdownBlock> {
        val blocks = mutableListOf<MarkdownBlock>()
        val codeBlockRegex = Regex("""```(\w*)?([\s\S]*?)```""")
        
        var lastIndex = 0
        codeBlockRegex.findAll(content).forEach { matchResult ->
            // Add text block before the code block
            if (matchResult.range.first > lastIndex) {
                val text = content.substring(lastIndex, matchResult.range.first)
                blocks.add(MarkdownBlock.Text(parseInlineFormatting(text)))
            }
            
            val language = matchResult.groupValues[1].ifBlank { "code" }
            val codeContent = matchResult.groupValues[2].trim()
            blocks.add(MarkdownBlock.Code(codeContent, language))
            
            lastIndex = matchResult.range.last + 1
        }
        
        // Add remaining text
        if (lastIndex < content.length) {
            val text = content.substring(lastIndex)
            blocks.add(MarkdownBlock.Text(parseInlineFormatting(text)))
        }
        
        return if (blocks.isEmpty()) listOf(MarkdownBlock.Text(parseInlineFormatting(content))) else blocks
    }

    private fun parseInlineFormatting(text: String): AnnotatedString {
        return buildAnnotatedString {
            var currentText = text
            
            // This is a simple parser. For production, consider a more robust state-machine based approach.
            // Support for **bold**
            val boldRegex = Regex("""\*\*(.*?)\*\*""")
            // Support for `inline code`
            val inlineCodeRegex = Regex("""`(.*?)`""")
            
            val spans = mutableListOf<Triple<IntRange, SpanStyle, String>>()
            
            boldRegex.findAll(currentText).forEach { match ->
                spans.add(Triple(match.range, SpanStyle(fontWeight = FontWeight.Bold), match.groupValues[1]))
            }
            
            inlineCodeRegex.findAll(currentText).forEach { match ->
                spans.add(Triple(match.range, SpanStyle(
                    fontFamily = FontFamily.Monospace,
                    background = Color(0xFF1e293b), // Slate-800
                    color = Color(0xFF38bdf8) // Cyan-400
                ), match.groupValues[1]))
            }
            
            // Sort spans by their start position to process them
            val sortedSpans = spans.sortedBy { it.first.first }
            
            var lastPos = 0
            sortedSpans.forEach { (range, style, innerText) ->
                // Overlap check
                if (range.first >= lastPos) {
                    append(currentText.substring(lastPos, range.first))
                    pushStyle(style)
                    append(innerText)
                    pop()
                    lastPos = range.last + 1
                }
            }
            append(currentText.substring(lastPos))
        }
    }
}
