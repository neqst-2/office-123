import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/rpc/local_rpc_client.dart';

class CalendarPlaceholder extends ConsumerWidget {
  const CalendarPlaceholder({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    final now = DateTime.now();
    final range = AgendaRange(start: now, end: now.add(const Duration(hours: 24)));
    final agendaAsync = ref.watch(agendaProvider(range));

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
        child: fluent.Column(
          crossAxisAlignment: fluent.CrossAxisAlignment.start,
          children: <fluent.Widget>[
            fluent.Text(
              'Agenda rendering, scheduling lanes, and recurrence tooling arrive next.',
              style: fluent.FluentTheme.of(context).typography.body,
            ),
            const fluent.SizedBox(height: 16),
            agendaAsync.when(
              data: (final items) {
                if (items.isEmpty) {
                  return const fluent.Text('No events in the next 24h (or RPC offline).');
                }
                final preview = items.take(5).toList(growable: false);
                return fluent.Column(
                  crossAxisAlignment: fluent.CrossAxisAlignment.start,
                  children: <fluent.Widget>[
                    fluent.Text(
                      'Agenda (preview)',
                      style: fluent.FluentTheme.of(context).typography.subtitle,
                    ),
                    const fluent.SizedBox(height: 8),
                    for (final item in preview)
                      fluent.Text(
                        '${item['title'] ?? '(no title)'}',
                        style: fluent.FluentTheme.of(context).typography.body,
                      ),
                  ],
                );
              },
              error: (final err, final st) => fluent.Text('RPC error: $err'),
              loading: () => const fluent.ProgressRing(),
            ),
          ],
        ),
      ),
    );
  }
}
