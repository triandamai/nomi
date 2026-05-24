import 'package:flutter/material.dart';

class NomiAvatar extends StatelessWidget {
  final String name;
  final bool active;
  final bool online;
  final double size;
  final VoidCallback? onTap;

  const NomiAvatar({
    super.key,
    required this.name,
    this.active = false,
    this.online = false,
    this.size = 48,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final String initial = name.isNotEmpty ? name[0].toUpperCase() : '?';
    
    return GestureDetector(
      onTap: onTap,
      child: Container(
        width: size,
        height: size,
        decoration: BoxDecoration(
          color: active ? const Color(0xFF3b82f6) : const Color(0xFF1e293b),
          borderRadius: BorderRadius.circular(active ? 16 : size / 2),
          boxShadow: active ? [
            BoxShadow(
              color: const Color(0xFF3b82f6).withValues(alpha: 0.3),
              blurRadius: 10,
              offset: const Offset(0, 4),
            )
          ] : [],
        ),
        child: Stack(
          children: [
            Center(
              child: Text(
                initial,
                style: TextStyle(
                  color: Colors.white,
                  fontSize: size * 0.4,
                  fontWeight: FontWeight.w900,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            if (online)
              Positioned(
                right: 2,
                bottom: 2,
                child: Container(
                  width: size * 0.25,
                  height: size * 0.25,
                  decoration: BoxDecoration(
                    color: const Color(0xFF10b981),
                    shape: BoxShape.circle,
                    border: Border.all(color: const Color(0xFF020617), width: 2),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
