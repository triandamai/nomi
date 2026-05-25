import 'package:intl/intl.dart';
import 'dart:math' as math;

class Formatter {
  static String formatTokenCount(dynamic tokens) {
    if (tokens == null) return '0';
    
    int value;
    if (tokens is String) {
      value = int.tryParse(tokens) ?? 0;
    } else if (tokens is num) {
      value = tokens.toInt();
    } else {
      return '0';
    }

    if (value >= 10000000) {
      final suffixes = ['', 'K', 'M', 'B', 'T'];
      final suffixNum = (value.toString().length - 1) ~/ 3;
      double shortValue = value / math.pow(1000, suffixNum);
      
      String formattedValue;
      if (shortValue % 1 != 0) {
        formattedValue = shortValue.toStringAsFixed(1);
      } else {
        formattedValue = shortValue.toInt().toString();
      }
      
      return formattedValue + suffixes[suffixNum];
    }
    
    // Standard format with dot separator (de-DE style logic)
    final formatter = NumberFormat('#,###', 'de_DE');
    return formatter.format(value);
  }

  static String formatBytes(int bytes) {
    if (bytes <= 0) return "0 B";
    const suffixes = ["B", "KB", "MB", "GB", "TB"];
    var i = (math.log(bytes) / math.log(1024)).floor();
    return '${(bytes / math.pow(1024, i)).toStringAsFixed(2)} ${suffixes[i]}';
  }
}
