<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/rpc/local_rpc_client.dart';
import '../../../core/workspace_tabs.dart';

final _viewRawDatabaseStateProvider = StateProvider<bool>((final ref) => false);

/// Hosts the placeholder workspace for the future JMAP mail client.
class MailPlaceholder extends ConsumerWidget {
  /// Creates the mail placeholder widget.
  const MailPlaceholder({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    final mailsAsync = ref.watch(unreadMailsProvider);
    return _FeaturePlaceholder(
      title: 'Mail Workspace',
      subtitle: 'Thread list, conversation pane, and secure message actions arrive next.',
      action: fluent.FilledButton(
        child: const fluent.Text('Extract attachments and open as Linked Spreadsheet'),
        onPressed: () {
          ref.read(workspaceTabsProvider.notifier).openLinkedSpreadsheetFromMail(
                contextAnchor: 'linked_to:mail->document_meta#sheet:cell=R12C4',
              );
        },
      ),
      extra: mailsAsync.when(
        data: (final items) {
          if (items.isEmpty) {
            return const fluent.Text('No unread mails (or RPC offline).');
          }
          final bool viewRawDb = ref.watch(_viewRawDatabaseStateProvider);
          final preview = items.take(5).toList(growable: false);
          return fluent.Column(
            crossAxisAlignment: fluent.CrossAxisAlignment.start,
            children: <fluent.Widget>[
              const fluent.SizedBox(height: 16),
              fluent.Text(
                'Unread (preview)',
                style: fluent.FluentTheme.of(context).typography.subtitle,
              ),
              const fluent.SizedBox(height: 8),
              fluent.ToggleSwitch(
                checked: viewRawDb,
                content: const fluent.Text('View Raw Database State'),
                onChanged: (final value) {
                  ref.read(_viewRawDatabaseStateProvider.notifier).state = value;
                },
              ),
              const fluent.SizedBox(height: 12),
              for (final item in preview)
                _MailPreviewRow(
                  subject: '${item['subject'] ?? '(no subject)'}',
                  isE2ee: item['e2ee_protected'] == true,
                  previewText: _selectPreviewText(
                    item: item,
                    viewRawDb: viewRawDb,
                  ),
                ),
            ],
          );
        },
        error: (final err, final st) => fluent.Text('RPC error: $err'),
        loading: () => const fluent.Padding(
          padding: fluent.EdgeInsets.only(top: 16),
          child: fluent.ProgressRing(),
        ),
      ),
    );
  }
}

class _MailPreviewRow extends fluent.StatelessWidget {
  const _MailPreviewRow({
    required this.subject,
    required this.isE2ee,
    required this.previewText,
  });

  final String subject;
  final bool isE2ee;
  final String previewText;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.Container(
      margin: const fluent.EdgeInsets.only(bottom: 10),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          fluent.Text(
            subject,
            style: fluent.FluentTheme.of(context).typography.bodyStrong,
          ),
          if (isE2ee)
            const fluent.Padding(
              padding: fluent.EdgeInsets.only(top: 4),
              child: fluent.Text('🔒 E2EE Protected (Decrypted Locally via Rust Core via ML-KEM/X25519)'),
            ),
          fluent.Padding(
            padding: const fluent.EdgeInsets.only(top: 4),
            child: fluent.Text(
              previewText,
              style: fluent.FluentTheme.of(context).typography.caption,
            ),
          ),
        ],
      ),
    );
  }
}

String _selectPreviewText({
  required final Map<String, Object?> item,
  required final bool viewRawDb,
}) {
  final String? decrypted = item['body_text_plain'] as String?;
  final String? raw = item['body_text_raw'] as String?;

  final String selected = viewRawDb
      ? (raw ?? decrypted ?? '')
      : (decrypted ?? raw ?? '');

  if (selected.isEmpty) {
    return '(no body preview)';
  }

  const int maxLen = 120;
  if (selected.length <= maxLen) {
    return selected;
  }
  return '${selected.substring(0, maxLen)}…';
}

/// Renders a reusable feature placeholder panel.
class _FeaturePlaceholder extends fluent.StatelessWidget {
  /// Creates a standardized placeholder panel.
  const _FeaturePlaceholder({
    required this.title,
    required this.subtitle,
    required this.action,
    required this.extra,
  });

  /// Heading shown in the content pane.
  final String title;

  /// Descriptive text shown below the heading.
  final String subtitle;

  final fluent.Widget action;
  final fluent.Widget extra;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.InfoLabel(
      label: title,
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
              subtitle,
              style: fluent.FluentTheme.of(context).typography.body,
            ),
            const fluent.SizedBox(height: 16),
            action,
            extra,
          ],
        ),
      ),
    );
  }
}
=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/workspace_tabs.dart';

/// Hosts the placeholder workspace for the future JMAP mail client.
class MailPlaceholder extends ConsumerWidget {
  /// Creates the mail placeholder widget.
  const MailPlaceholder({super.key});

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    return _FeaturePlaceholder(
      title: 'Mail Workspace',
      subtitle: 'Thread list, conversation pane, and secure message actions arrive next.',
      action: fluent.FilledButton(
        child: const fluent.Text('Extract attachments and open as Linked Spreadsheet'),
        onPressed: () {
          ref.read(workspaceTabsProvider.notifier).openLinkedSpreadsheetFromMail(
                contextAnchor: 'linked_to:mail->document_meta#sheet:cell=R12C4',
              );
        },
      ),
    );
  }
}

/// Renders a reusable feature placeholder panel.
class _FeaturePlaceholder extends fluent.StatelessWidget {
  /// Creates a standardized placeholder panel.
  const _FeaturePlaceholder({
    required this.title,
    required this.subtitle,
    required this.action,
  });

  /// Heading shown in the content pane.
  final String title;

  /// Descriptive text shown below the heading.
  final String subtitle;

  final fluent.Widget action;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.InfoLabel(
      label: title,
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
              subtitle,
              style: fluent.FluentTheme.of(context).typography.body,
            ),
            const fluent.SizedBox(height: 16),
            action,
          ],
        ),
      ),
    );
  }
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
