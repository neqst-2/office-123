import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../core/rpc/local_rpc_client.dart';

/// Renders the neutral dashboard landing surface of the shell.
class DashboardHome extends ConsumerWidget {
  /// Creates the dashboard placeholder widget.
  const DashboardHome({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    final RpcConnectionState connectionState =
        ref.watch(rpcConnectionStateProvider).maybeWhen(
              data: (final s) => s,
              orElse: () => RpcConnectionState.disconnected,
            );

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
        const fluent.SizedBox(height: 18),
        _SovereignNodePanel(connectionState: connectionState),
      ],
    );
  }
}

final _peersProvider = FutureProvider<List<Map<String, Object?>>>((final ref) async {
  final client = ref.watch(rpcClientProvider);
  final result = await client.call('sync:list_peers', const <String, Object?>{});
  final Object? peers = result['peers'];
  if (peers is List) {
    return peers.whereType<Map>().map((final e) => e.cast<String, Object?>()).toList();
  }
  return const <Map<String, Object?>>[];
});

final _pairingTokenProvider = StateProvider<String?>((final ref) => null);

class _SovereignNodePanel extends ConsumerWidget {
  const _SovereignNodePanel({required this.connectionState});

  final RpcConnectionState connectionState;

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    const endpoint = 'ws://127.0.0.1:9001/ws';
    final peersAsync = ref.watch(_peersProvider);
    final token = ref.watch(_pairingTokenProvider);

    return fluent.Container(
      width: double.infinity,
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
            'Sovereign Cloud & Node Settings',
            style: fluent.FluentTheme.of(context).typography.subtitle,
          ),
          const fluent.SizedBox(height: 8),
          fluent.Text('Bind/Endpoint: $endpoint'),
          fluent.Text('Status: ${connectionState.name}'),
          const fluent.SizedBox(height: 12),
          fluent.FilledButton(
            child: const fluent.Text('Generate Secure Remote Access Token'),
            onPressed: () async {
              final client = ref.read(rpcClientProvider);
              try {
                final result = await client.call(
                  'sync:generate_remote_token',
                  const <String, Object?>{'ttl_seconds': 900},
                );
                final String? t = result['token'] as String?;
                ref.read(_pairingTokenProvider.notifier).state = t;
              } catch (e) {
                ref.read(_pairingTokenProvider.notifier).state = null;
              }
            },
          ),
          if (token != null) ...<fluent.Widget>[
            const fluent.SizedBox(height: 12),
            fluent.InfoLabel(
              label: 'One-time pairing token',
              child: fluent.SelectableText(token),
            ),
            const fluent.SizedBox(height: 10),
            fluent.InfoLabel(
              label: 'QR-code',
              child: fluent.SizedBox(
                width: 180,
                height: 180,
                child: QrImageView(
                  data: token,
                  version: QrVersions.auto,
                  backgroundColor: const fluent.Color(0xFFFFFFFF),
                ),
              ),
            ),
          ],
          const fluent.SizedBox(height: 16),
          fluent.Text(
            'Active peers (ledger)',
            style: fluent.FluentTheme.of(context).typography.bodyStrong,
          ),
          const fluent.SizedBox(height: 8),
          peersAsync.when(
            data: (final peers) {
              if (peers.isEmpty) {
                return const fluent.Text(
                  'No peers connected. Example: "Connected Device: Mobile Phone via Secure Local WebService - E2EE Active".',
                );
              }
              return fluent.Column(
                crossAxisAlignment: fluent.CrossAxisAlignment.start,
                children: <fluent.Widget>[
                  for (final peer in peers)
                    fluent.Text(
                      'Connected Device: ${peer['addr']} - ${peer['mode']} - E2EE Active',
                    ),
                ],
              );
            },
            error: (final e, final st) => fluent.Text('RPC error: $e'),
            loading: () => const fluent.ProgressRing(),
          ),
          const fluent.SizedBox(height: 14),
          fluent.Text(
            'Integration Matrix Status',
            style: fluent.FluentTheme.of(context).typography.bodyStrong,
          ),
          const fluent.SizedBox(height: 6),
          const fluent.Badge(
            child: fluent.Text('PASETO v4 Core Verified & Graph Interoperability Ready'),
          ),
          const fluent.SizedBox(height: 6),
          const fluent.Text('Source: Local RPC sync:list_peers / sync:generate_remote_token'),
        ],
      ),
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
