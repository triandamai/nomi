import 'dart:math' as math;
import 'package:flutter/material.dart';

class TypingIndicator extends StatefulWidget {
  final Color color;
  const TypingIndicator({super.key, this.color = Colors.blue});

  @override
  State<TypingIndicator> createState() => _TypingIndicatorState();
}

class _TypingIndicatorState extends State<TypingIndicator> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(3, (index) {
          return AnimatedBuilder(
            animation: _animationController,
            builder: (context, child) {
              // Offset sine wave for each dot to create a "wave" effect
              final double delay = index * 0.4;
              final double value = math.sin((_animationController.value * 2 * math.pi) - delay);
              final double bounce = (value + 1) / 2 * 4; // Bouncing height

              return Container(
                margin: const EdgeInsets.symmetric(horizontal: 2),
                transform: Matrix4.translationValues(0, -bounce, 0),
                width: 4,
                height: 4,
                decoration: BoxDecoration(
                  color: widget.color.withValues(alpha: (value + 1) / 2 * 0.5 + 0.3),
                  shape: BoxShape.circle,
                ),
              );
            },
          );
        }),
      ),
    );
  }
}
