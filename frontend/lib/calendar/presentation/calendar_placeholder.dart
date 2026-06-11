import 'package:fluent_ui/fluent_ui.dart' as fluent;

/// Hosts the placeholder workspace for the future CalDAV calendar client.
class CalendarPlaceholder extends fluent.StatelessWidget {
  /// Creates the calendar placeholder widget.
  const CalendarPlaceholder({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.InfoLabel(
      label: 'Calendar Workspace',
      child: fluent.Container(
        width: double.infinity,
        padding: const fluent.EdgeInsets.all(18),
        decoration: fluent.BoxDecoration(
          color: fluent.FluentTheme.of(context).micaBackgroundColor,
          borderRadius: fluent.BorderRadius.circular(18),
          border: fluent.Border.all(
            color: fluent.FluentTheme.of(context).inactiveBackgroundColor,
          ),
        ),
        child: fluent.Text(
          'Agenda rendering, scheduling lanes, and recurrence tooling arrive next.',
          style: fluent.FluentTheme.of(context).typography.body,
        ),
      ),
    );
  }
}
