import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';

class CodeBlock extends StatefulWidget {
  final String code;
  final String language;

  const CodeBlock({
    super.key,
    required this.code,
    this.language = 'text',
  });

  @override
  State<CodeBlock> createState() => _CodeBlockState();
}

class _CodeBlockState extends State<CodeBlock> {
  bool _isExpanded = false;
  bool _isCopied = false;

  @override
  Widget build(BuildContext context) {
    // 🎨 Discord/Artifact Style Palette
    const Color headerBg = Color(0xFF202225);
    const Color codeBg = Color(0xFF2f3136);
    const Color accentColor = Color(0xFF3b82f6);

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: codeBg,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // 🏷️ Header with Language, Expand, and Copy
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: const BoxDecoration(
              color: headerBg,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    const Icon(LucideIcons.code2, size: 14, color: Colors.white24),
                    const SizedBox(width: 8),
                    Text(
                      widget.language.toUpperCase(),
                      style: const TextStyle(
                        color: Colors.white54,
                        fontSize: 10,
                        fontWeight: FontWeight.w900,
                        letterSpacing: 1.2,
                      ),
                    ),
                  ],
                ),
                Row(
                  children: [
                    // Expand Toggle
                    _HeaderButton(
                      icon: _isExpanded ? LucideIcons.chevronUp : LucideIcons.chevronDown,
                      onTap: () => setState(() => _isExpanded = !_isExpanded),
                    ),
                    const SizedBox(width: 8),
                    // Copy Button
                    _HeaderButton(
                      icon: _isCopied ? LucideIcons.check : LucideIcons.copy,
                      color: _isCopied ? const Color(0xFF10b981) : Colors.white24,
                      onTap: _handleCopy,
                    ),
                  ],
                ),
              ],
            ),
          ),

          // 💻 Code Content
          Container(
            padding: const EdgeInsets.all(16),
            constraints: BoxConstraints(
              maxHeight: _isExpanded ? double.infinity : 200,
            ),
            child: SingleChildScrollView(
              physics: _isExpanded ? const NeverScrollableScrollPhysics() : const ClampingScrollPhysics(),
              child: Text(
                widget.code,
                style: const TextStyle(
                  color: Color(0xFFdcddde),
                  fontFamily: 'monospace',
                  fontSize: 12,
                  height: 1.5,
                ),
              ),
            ),
          ),
          
          // Show More Gradient (if collapsed and long)
          if (!_isExpanded && widget.code.split('\n').length > 5)
            GestureDetector(
              onTap: () => setState(() => _isExpanded = true),
              child: Container(
                height: 40,
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                    colors: [
                      codeBg.withValues(alpha: 0),
                      codeBg.withValues(alpha: 0.8),
                    ],
                  ),
                ),
                child: const Center(
                  child: Text(
                    'SHOW MORE',
                    style: TextStyle(
                      color: accentColor,
                      fontSize: 8,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 1,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Future<void> _handleCopy() async {
    await Clipboard.setData(ClipboardData(text: widget.code));
    setState(() => _isCopied = true);
    Future.delayed(const Duration(seconds: 2), () {
      if (mounted) setState(() => _isCopied = false);
    });
  }
}

class _HeaderButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback onTap;
  final Color? color;

  const _HeaderButton({required this.icon, required this.onTap, this.color});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.all(4),
        child: Icon(icon, size: 14, color: color ?? Colors.white24),
      ),
    );
  }
}
