<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/rpc/local_rpc_client.dart';

enum DocumentTaskPhase {
  idle,
  queued,
  started,
  processing,
  completed,
  failed,
}

final class DocumentTaskState {
  const DocumentTaskState({
    required this.phase,
    required this.taskId,
    required this.percentage,
    required this.statusMessage,
    required this.logs,
    required this.result,
    required this.errorMessage,
  });

  const DocumentTaskState.idle()
      : phase = DocumentTaskPhase.idle,
        taskId = null,
        percentage = 0,
        statusMessage = '',
        logs = const <String>[],
        result = null,
        errorMessage = null;

  final DocumentTaskPhase phase;
  final String? taskId;
  final int percentage;
  final String statusMessage;
  final List<String> logs;
  final Map<String, Object?>? result;
  final String? errorMessage;

  DocumentTaskState copyWith({
    final DocumentTaskPhase? phase,
    final String? taskId,
    final int? percentage,
    final String? statusMessage,
    final List<String>? logs,
    final Map<String, Object?>? result,
    final String? errorMessage,
  }) {
    return DocumentTaskState(
      phase: phase ?? this.phase,
      taskId: taskId ?? this.taskId,
      percentage: percentage ?? this.percentage,
      statusMessage: statusMessage ?? this.statusMessage,
      logs: logs ?? this.logs,
      result: result ?? this.result,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

final _documentTaskControllerProvider =
    StateNotifierProvider<_DocumentTaskController, DocumentTaskState>((final ref) {
  final client = ref.watch(rpcClientProvider);
  final controller = _DocumentTaskController(client);
  ref.onDispose(controller.dispose);
  return controller;
});

final class _DocumentTaskController extends StateNotifier<DocumentTaskState> {
  _DocumentTaskController(this._client) : super(const DocumentTaskState.idle()) {
    _subscription = _client.notifications.listen(_handleNotification);
  }

  final NeqstRpcClient _client;
  StreamSubscription<RpcNotification>? _subscription;

  Future<void> open({
    required final String path,
    final String? contextAnchor,
  }) async {
    state = const DocumentTaskState(
      phase: DocumentTaskPhase.queued,
      taskId: null,
      percentage: 0,
      statusMessage: 'Queued',
      logs: <String>['Queued document processing task...'],
      result: null,
      errorMessage: null,
    );

    try {
      final Map<String, Object?> params = <String, Object?>{'path': path};
      if (contextAnchor != null) {
        params['link'] = <String, Object?>{
          'from': 'mail:demo',
          'context_anchor': contextAnchor,
        };
      }

      final result = await _client.call('doc:open_document', params);
      final String? taskId = result['task_id'] as String?;
      state = state.copyWith(
        phase: DocumentTaskPhase.queued,
        taskId: taskId,
        statusMessage: 'Queued',
        logs: <String>[...state.logs, 'Task ID: ${taskId ?? '(missing)'}'],
      );
    } catch (e) {
      state = state.copyWith(
        phase: DocumentTaskPhase.failed,
        errorMessage: e.toString(),
        logs: <String>[...state.logs, 'Failed to queue: $e'],
      );
    }
  }

  void dispose() {
    _subscription?.cancel();
    _subscription = null;
    super.dispose();
  }

  void _handleNotification(final RpcNotification n) {
    if (n.method != 'task:progress') {
      return;
    }
    final String? currentTask = state.taskId;
    if (currentTask == null) {
      return;
    }

    final Object? progressObj = n.params['progress'];
    if (progressObj is! Map) {
      return;
    }

    final Map<String, Object?> progress = progressObj.cast<String, Object?>();
    if (progress.containsKey('Started')) {
      final data = (progress['Started'] as Map?)?.cast<String, Object?>();
      if (data?['task_id'] != currentTask) {
        return;
      }
      state = state.copyWith(
        phase: DocumentTaskPhase.started,
        percentage: 1,
        statusMessage: 'Started',
        logs: <String>[...state.logs, 'Started'],
      );
      return;
    }

    if (progress.containsKey('Processing')) {
      final data = (progress['Processing'] as Map?)?.cast<String, Object?>();
      if (data?['task_id'] != currentTask) {
        return;
      }
      final int pct = (data?['percentage'] as int?) ?? state.percentage;
      final String msg = (data?['status_message'] as String?) ?? '';
      state = state.copyWith(
        phase: DocumentTaskPhase.processing,
        percentage: pct,
        statusMessage: msg,
        logs: msg.isEmpty ? state.logs : <String>[...state.logs, msg],
      );
      return;
    }

    if (progress.containsKey('Completed')) {
      final data = (progress['Completed'] as Map?)?.cast<String, Object?>();
      if (data?['task_id'] != currentTask) {
        return;
      }
      final Object? resultObj = n.params['result'];
      state = state.copyWith(
        phase: DocumentTaskPhase.completed,
        percentage: 100,
        statusMessage: (data?['result_summary'] as String?) ?? 'Completed',
        result: resultObj is Map ? resultObj.cast<String, Object?>() : state.result,
        logs: <String>[...state.logs, 'Completed'],
      );
      return;
    }

    if (progress.containsKey('Failed')) {
      final data = (progress['Failed'] as Map?)?.cast<String, Object?>();
      if (data?['task_id'] != currentTask) {
        return;
      }
      final String err = (data?['error_message'] as String?) ?? 'unknown_error';
      state = state.copyWith(
        phase: DocumentTaskPhase.failed,
        errorMessage: err,
        statusMessage: err,
        logs: <String>[...state.logs, 'Failed: $err'],
      );
    }
  }
}

class DocumentsPlaceholder extends ConsumerStatefulWidget {
  const DocumentsPlaceholder({
    super.key,
    this.contextAnchor,
  });

  final String? contextAnchor;

  @override
  ConsumerState<DocumentsPlaceholder> createState() => _DocumentsPlaceholderState();
}

class _DocumentsPlaceholderState extends ConsumerState<DocumentsPlaceholder> {
  late final fluent.TextEditingController _pathController;

  @override
  void initState() {
    super.initState();
    _pathController = fluent.TextEditingController(text: 'docs/demo.ods');
  }

  @override
  void dispose() {
    _pathController.dispose();
    super.dispose();
  }

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    final taskState = ref.watch(_documentTaskControllerProvider);

    return fluent.InfoLabel(
      label: 'Document Workspace',
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
              'Docking panels for the LibreOffice renderer and metadata overlays arrive next.',
              style: fluent.FluentTheme.of(context).typography.body,
            ),
            const fluent.SizedBox(height: 16),
            fluent.TextBox(
              header: 'storage_path',
              placeholder: 'relative/path/to/file.odt',
              controller: _pathController,
            ),
            const fluent.SizedBox(height: 10),
            fluent.FilledButton(
              child: const fluent.Text('Open via Local RPC'),
              onPressed: () {
                ref.read(_documentTaskControllerProvider.notifier).open(
                      path: _pathController.text,
                      contextAnchor: widget.contextAnchor,
                    );
              },
            ),
            if (widget.contextAnchor != null) ...<fluent.Widget>[
              const fluent.SizedBox(height: 12),
              fluent.InfoLabel(
                label: 'context_anchor',
                child: fluent.Text(
                  widget.contextAnchor!,
                  style: fluent.FluentTheme.of(context).typography.caption,
                ),
              ),
            ],
            const fluent.SizedBox(height: 16),
            _TaskProgressPanel(state: taskState),
          ],
        ),
      ),
    );
  }
}

class _TaskProgressPanel extends fluent.StatelessWidget {
  const _TaskProgressPanel({required this.state});

  final DocumentTaskState state;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    if (state.phase == DocumentTaskPhase.idle) {
      return const fluent.Text('No document opened yet.');
    }

    final Map<String, Object?>? result = state.result;
    final Map<String, Object?>? meta = (result?['meta'] as Map?)?.cast<String, Object?>();
    final Map<String, Object?>? lok = (result?['lok'] as Map?)?.cast<String, Object?>();
    final bool isGraph = result?['is_graph_enhanced'] == true;

    return fluent.Container(
      padding: const fluent.EdgeInsets.all(14),
      decoration: fluent.BoxDecoration(
        color: fluent.FluentTheme.of(context).acrylicBackgroundColor,
        borderRadius: fluent.BorderRadius.circular(16),
        border: fluent.Border.all(
          color: fluent.FluentTheme.of(context).inactiveBackgroundColor,
        ),
      ),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          fluent.Text(
            'Background Task',
            style: fluent.FluentTheme.of(context).typography.subtitle,
          ),
          const fluent.SizedBox(height: 8),
          fluent.Text('task_id: ${state.taskId ?? '-'}'),
          fluent.Text('phase: ${state.phase.name}'),
          if (state.statusMessage.isNotEmpty) fluent.Text('status: ${state.statusMessage}'),
          const fluent.SizedBox(height: 10),
          fluent.ProgressBar(value: state.percentage / 100.0),
          const fluent.SizedBox(height: 12),
          fluent.InfoLabel(
            label: 'Live worker log',
            child: fluent.Container(
              width: double.infinity,
              padding: const fluent.EdgeInsets.all(10),
              decoration: fluent.BoxDecoration(
                color: fluent.FluentTheme.of(context).micaBackgroundColor,
                borderRadius: fluent.BorderRadius.circular(12),
              ),
              child: fluent.Column(
                crossAxisAlignment: fluent.CrossAxisAlignment.start,
                children: <fluent.Widget>[
                  for (final line in state.logs.take(8)) fluent.Text(line),
                ],
              ),
            ),
          ),
          if (state.phase == DocumentTaskPhase.failed && state.errorMessage != null) ...<fluent.Widget>[
            const fluent.SizedBox(height: 10),
            fluent.Text('error: ${state.errorMessage}'),
          ],
          if (state.phase == DocumentTaskPhase.completed && meta != null) ...<fluent.Widget>[
            const fluent.SizedBox(height: 14),
            fluent.Text(
              'Document Metadata',
              style: fluent.FluentTheme.of(context).typography.subtitle,
            ),
            const fluent.SizedBox(height: 8),
            fluent.Text('filename: ${meta['filename'] ?? '-'}'),
            fluent.Text('storage_path: ${meta['storage_path'] ?? '-'}'),
            fluent.Text('file_size: ${meta['file_size'] ?? '-'}'),
            fluent.Text('mime_type: ${meta['mime_type'] ?? '-'}'),
            const fluent.SizedBox(height: 10),
            fluent.Text('parts/pages: ${lok?['parts'] ?? 0}'),
            fluent.Text('lok_available: ${lok?['available'] ?? false}'),
            const fluent.SizedBox(height: 10),
            fluent.Badge(
              child: fluent.Text(
                isGraph ? 'Graph-Enhanced Mode (ODF)' : 'Compatibility Mode (OOXML/Other)',
              ),
            ),
          ],
        ],
      ),
    );
  }
}
=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;

/// Hosts the placeholder workspace for the future LibreOffice document shell.
class DocumentsPlaceholder extends fluent.StatelessWidget {
  /// Creates the documents placeholder widget.
  const DocumentsPlaceholder({
    super.key,
    this.contextAnchor,
  });

  final String? contextAnchor;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.InfoLabel(
      label: 'Document Workspace',
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
              'Docking panels for the LibreOffice renderer and metadata overlays arrive next.',
              style: fluent.FluentTheme.of(context).typography.body,
            ),
            if (contextAnchor != null) ...<fluent.Widget>[
              const fluent.SizedBox(height: 12),
              fluent.InfoLabel(
                label: 'context_anchor',
                child: fluent.Text(
                  contextAnchor!,
                  style: fluent.FluentTheme.of(context).typography.caption,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
