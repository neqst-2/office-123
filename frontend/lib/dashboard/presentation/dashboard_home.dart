import 'package:fluent_ui/fluent_ui.dart' as fluent;

/// Renders the neutral dashboard landing surface of the shell.
class DashboardHome extends fluent.StatelessWidget {
  /// Creates the dashboard placeholder widget.
  const DashboardHome({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.Column(
      crossAxisAlignment: fluent.CrossAxisAlignment.start,
      children: <fluent.Widget>[
        fluent.Text(
          'Lotus-Moderne Workspace',
          style: fluent.FluentTheme.of(context).typography.titleLarge,
        ),
        const fluent.SizedBox(height: 8),
        fluent.Text(
          'Unified launch surface for Mail, Calendar, Documents, and future data views.',
          style: fluent.FluentTheme.of(context).typography.body,
        ),
        const fluent.SizedBox(height: 24),
        fluent.Wrap(
          spacing: 12,
          runSpacing: 12,
          children: const <fluent.Widget>[
            _StatusCard(
              title: 'Mail',
              subtitle: 'JMAP inbox and thread workspace placeholder',
            ),
            _StatusCard(
              title: 'Calendar',
              subtitle: 'CalDAV agenda and schedule grid placeholder',
            ),
            _StatusCard(
              title: 'Documents',
              subtitle: 'LibreOffice docking surface placeholder',
            ),
          ],
        ),
      ],
    );
  }
}

/// Displays a compact summary tile on the dashboard.
class _StatusCard extends fluent.StatelessWidget {
  /// Creates a compact dashboard status card.
  const _StatusCard({
    required this.title,
    required this.subtitle,
  });

  /// Primary heading of the tile.
  final String title;

  /// Secondary supporting text of the tile.
  final String subtitle;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.Container(
      width: 260,
      padding: const fluent.EdgeInsets.all(16),
      decoration: fluent.BoxDecoration(
        color: fluent.FluentTheme.of(context).micaBackgroundColor,
        borderRadius: fluent.BorderRadius.circular(16),
        border: fluent.Border.all(
          color: fluent.FluentTheme.of(context).inactiveBackgroundColor,
        ),
      ),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          fluent.Text(
            title,
            style: fluent.FluentTheme.of(context).typography.subtitle,
          ),
          const fluent.SizedBox(height: 8),
          fluent.Text(
            subtitle,
            style: fluent.FluentTheme.of(context).typography.caption,
          ),
        ],
      ),
    );
  }
}
