<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter/material.dart' as material;

/// Bundles the shell theme definitions for both Fluent and Material widgets.
final class AppThemeBundle {
  /// Creates a theme bundle for the current visual mode.
  const AppThemeBundle({
    required this.fluentTheme,
    required this.materialTheme,
  });

  /// Fluent desktop theme used by the root shell.
  final fluent.FluentThemeData fluentTheme;

  /// Material 3 theme exposed to embedded Material components.
  final material.ThemeData materialTheme;
}

/// Builds the design system palette for the requested theme mode.
///
/// Intent:
/// Returns a synchronized Fluent + Material theme pair so desktop and web stay
/// visually aligned while preserving native control semantics.
///
/// Input/Output constraints:
/// Accepts a Fluent [fluent.ThemeMode] and returns a fully built bundle.
///
/// Security implications:
/// This function must remain deterministic and side-effect free so that theme
/// changes do not trigger hidden I/O or leak host environment details.
AppThemeBundle buildAppTheme(final fluent.ThemeMode mode) {
  final bool isDarkMode = mode != fluent.ThemeMode.light;
  final material.ColorScheme materialScheme = material.ColorScheme.fromSeed(
    seedColor: const material.Color(0xFF5566FF),
    brightness:
        isDarkMode ? material.Brightness.dark : material.Brightness.light,
  );

  final fluent.AccentColor accentColor = fluent.AccentColor.swatch(
    const <String, fluent.Color>{
      'darkest': fluent.Color(0xFF14205C),
      'darker': fluent.Color(0xFF223588),
      'dark': fluent.Color(0xFF314AB2),
      'normal': fluent.Color(0xFF5566FF),
      'light': fluent.Color(0xFF7D8BFF),
      'lighter': fluent.Color(0xFFAAB5FF),
      'lightest': fluent.Color(0xFFD8DEFF),
    },
  );

  final fluent.FluentThemeData fluentTheme = fluent.FluentThemeData(
    brightness: isDarkMode ? fluent.Brightness.dark : fluent.Brightness.light,
    accentColor: accentColor,
    visualDensity: fluent.VisualDensity.standard,
    scaffoldBackgroundColor:
        isDarkMode ? const fluent.Color(0xFF111317) : const fluent.Color(0xFFF4F6FB),
    acrylicBackgroundColor:
        isDarkMode ? const fluent.Color(0xCC171A20) : const fluent.Color(0xD9FFFFFF),
  );

  final material.ThemeData materialTheme = material.ThemeData(
    useMaterial3: true,
    colorScheme: materialScheme,
    scaffoldBackgroundColor: materialScheme.surface,
    cardTheme: material.CardThemeData(
      color: materialScheme.surfaceContainerHigh,
      elevation: 0,
      shape: material.RoundedRectangleBorder(
        borderRadius: material.BorderRadius.circular(16),
      ),
    ),
  );

  return AppThemeBundle(
    fluentTheme: fluentTheme,
    materialTheme: materialTheme,
  );
}
=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter/material.dart' as material;

/// Bundles the shell theme definitions for both Fluent and Material widgets.
final class AppThemeBundle {
  /// Creates a theme bundle for the current visual mode.
  const AppThemeBundle({
    required this.fluentTheme,
    required this.materialTheme,
  });

  /// Fluent desktop theme used by the root shell.
  final fluent.FluentThemeData fluentTheme;

  /// Material 3 theme exposed to embedded Material components.
  final material.ThemeData materialTheme;
}

/// Builds the design system palette for the requested theme mode.
///
/// Intent:
/// Returns a synchronized Fluent + Material theme pair so desktop and web stay
/// visually aligned while preserving native control semantics.
///
/// Input/Output constraints:
/// Accepts a Fluent [fluent.ThemeMode] and returns a fully built bundle.
///
/// Security implications:
/// This function must remain deterministic and side-effect free so that theme
/// changes do not trigger hidden I/O or leak host environment details.
AppThemeBundle buildAppTheme(final fluent.ThemeMode mode) {
  final bool isDarkMode = mode != fluent.ThemeMode.light;
  final material.ColorScheme materialScheme = material.ColorScheme.fromSeed(
    seedColor: const material.Color(0xFF5566FF),
    brightness:
        isDarkMode ? material.Brightness.dark : material.Brightness.light,
  );

  final fluent.AccentColor accentColor = fluent.AccentColor.swatch(
    const <String, fluent.Color>{
      'darkest': fluent.Color(0xFF14205C),
      'darker': fluent.Color(0xFF223588),
      'dark': fluent.Color(0xFF314AB2),
      'normal': fluent.Color(0xFF5566FF),
      'light': fluent.Color(0xFF7D8BFF),
      'lighter': fluent.Color(0xFFAAB5FF),
      'lightest': fluent.Color(0xFFD8DEFF),
    },
  );

  final fluent.FluentThemeData fluentTheme = fluent.FluentThemeData(
    brightness: isDarkMode ? fluent.Brightness.dark : fluent.Brightness.light,
    accentColor: accentColor,
    visualDensity: fluent.VisualDensity.standard,
    scaffoldBackgroundColor:
        isDarkMode ? const fluent.Color(0xFF111317) : const fluent.Color(0xFFF4F6FB),
    acrylicBackgroundColor:
        isDarkMode ? const fluent.Color(0xCC171A20) : const fluent.Color(0xD9FFFFFF),
  );

  final material.ThemeData materialTheme = material.ThemeData(
    useMaterial3: true,
    colorScheme: materialScheme,
    scaffoldBackgroundColor: materialScheme.surface,
    cardTheme: material.CardThemeData(
      color: materialScheme.surfaceContainerHigh,
      elevation: 0,
      shape: material.RoundedRectangleBorder(
        borderRadius: material.BorderRadius.circular(16),
      ),
    ),
  );

  return AppThemeBundle(
    fluentTheme: fluentTheme,
    materialTheme: materialTheme,
  );
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
