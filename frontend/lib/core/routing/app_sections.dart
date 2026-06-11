<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;

/// Declares the primary workspaces that can be opened from the unified shell.
enum AppSection {
  /// Represents the cross-module dashboard overview.
  dashboard('Dashboard', fluent.FluentIcons.home),

  /// Represents the JMAP-oriented mail workspace.
  mail('Mail', fluent.FluentIcons.mail),

  /// Represents the CalDAV-oriented calendar workspace.
  calendar('Calendar', fluent.FluentIcons.calendar),

  /// Represents the LibreOffice-backed document workspace.
  documents('Documents', fluent.FluentIcons.document);

  /// Creates a strongly typed application section descriptor.
  const AppSection(this.label, this.iconData);

  /// Human-readable label used in shell navigation.
  final String label;

  /// Fluent icon used by the sidebar and tabs.
  final IconData iconData;
}
=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;

/// Declares the primary workspaces that can be opened from the unified shell.
enum AppSection {
  /// Represents the cross-module dashboard overview.
  dashboard('Dashboard', fluent.FluentIcons.home),

  /// Represents the JMAP-oriented mail workspace.
  mail('Mail', fluent.FluentIcons.mail),

  /// Represents the CalDAV-oriented calendar workspace.
  calendar('Calendar', fluent.FluentIcons.calendar),

  /// Represents the LibreOffice-backed document workspace.
  documents('Documents', fluent.FluentIcons.document);

  /// Creates a strongly typed application section descriptor.
  const AppSection(this.label, this.iconData);

  /// Human-readable label used in shell navigation.
  final String label;

  /// Fluent icon used by the sidebar and tabs.
  final IconData iconData;
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
