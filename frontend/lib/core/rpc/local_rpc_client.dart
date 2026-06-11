<<<<<<< HEAD
import 'dart:async';
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

enum RpcConnectionState {
  disconnected,
  connecting,
  connected,
}

final class NodeHealthSnapshot {
  const NodeHealthSnapshot({
    required this.dbStatus,
    required this.lokStatus,
    required this.lokVersion,
    required this.cryptoStatus,
    required this.queueBacklog,
    required this.activePeers,
  });

  final String dbStatus;
  final String lokStatus;
  final String lokVersion;
  final String cryptoStatus;
  final int queueBacklog;
  final int activePeers;

  factory NodeHealthSnapshot.fromJson(final Map<String, Object?> json) {
    return NodeHealthSnapshot(
      dbStatus: json['db_status'] as String? ?? 'initializing',
      lokStatus: json['lok_status'] as String? ?? 'initializing',
      lokVersion: json['lok_version'] as String? ?? 'unknown',
      cryptoStatus: json['crypto_status'] as String? ?? 'initializing',
      queueBacklog: (json['queue_backlog'] as num?)?.toInt() ?? 0,
      activePeers: (json['active_peers'] as num?)?.toInt() ?? 0,
    );
  }

  bool get isReady =>
      dbStatus == 'connected' &&
      lokStatus == 'ready' &&
      cryptoStatus == 'active';
}

final rpcNotificationsProvider = StreamProvider<RpcNotification>((final ref) {
  final client = ref.watch(rpcClientProvider);
  return client.notifications;
});

final class RpcNotification {
  const RpcNotification({
    required this.method,
    required this.params,
  });

  final String method;
  final Map<String, Object?> params;
}

final rpcClientProvider = Provider<NeqstRpcClient>((final ref) {
  final client = NeqstRpcClient(Uri.parse('ws://127.0.0.1:9001/ws'));
  ref.onDispose(client.dispose);
  return client;
});

final rpcConnectionStateProvider = StreamProvider<RpcConnectionState>((final ref) {
  final client = ref.watch(rpcClientProvider);
  return client.connectionStates;
});

final nodeHealthProvider = StreamProvider<NodeHealthSnapshot>((final ref) async* {
  final client = ref.watch(rpcClientProvider);
  while (true) {
    try {
      final result = await client.call('sys:get_node_status', const <String, Object?>{});
      yield NodeHealthSnapshot.fromJson(result);
    } catch (_) {
      yield const NodeHealthSnapshot(
        dbStatus: 'initializing',
        lokStatus: 'initializing',
        lokVersion: 'unknown',
        cryptoStatus: 'initializing',
        queueBacklog: 0,
        activePeers: 0,
      );
    }
    await Future<void>.delayed(const Duration(seconds: 3));
  }
});

final unreadMailsProvider = FutureProvider<List<Map<String, Object?>>>((final ref) async {
  final client = ref.watch(rpcClientProvider);
  final Map<String, Object?> result =
      await client.call('pim:get_unread_mails', <String, Object?>{'limit': 50});
  final Object? items = result['items'];
  if (items is List) {
    return items.whereType<Map>().map((final e) => e.cast<String, Object?>()).toList();
  }
  return const <Map<String, Object?>>[];
});

final agendaProvider =
    FutureProvider.family<List<Map<String, Object?>>, AgendaRange>((final ref, final range) async {
  final client = ref.watch(rpcClientProvider);
  final Map<String, Object?> result = await client.call(
    'pim:get_agenda',
    <String, Object?>{
      'start': range.start.toIso8601String(),
      'end': range.end.toIso8601String(),
    },
  );
  final Object? items = result['items'];
  if (items is List) {
    return items.whereType<Map>().map((final e) => e.cast<String, Object?>()).toList();
  }
  return const <Map<String, Object?>>[];
});

final class AgendaRange {
  const AgendaRange({
    required this.start,
    required this.end,
  });

  final DateTime start;
  final DateTime end;
}

final class NeqstRpcClient {
  NeqstRpcClient(
    this.endpoint, {
    this.baseBackoff = const Duration(milliseconds: 250),
    this.maxBackoff = const Duration(seconds: 15),
  }) {
    _connectLoop();
  }

  final Uri endpoint;
  final Duration baseBackoff;
  final Duration maxBackoff;

  final StreamController<RpcConnectionState> _stateController =
      StreamController<RpcConnectionState>.broadcast();

  final StreamController<RpcNotification> _notificationController =
      StreamController<RpcNotification>.broadcast();

  final Map<String, Completer<Map<String, Object?>>> _pending =
      <String, Completer<Map<String, Object?>>>{};

  WebSocketChannel? _channel;
  StreamSubscription<Object?>? _subscription;
  RpcConnectionState _state = RpcConnectionState.disconnected;
  bool _disposed = false;
  int _attempt = 0;
  int _idCounter = 0;

  Stream<RpcConnectionState> get connectionStates => _stateController.stream;
  Stream<RpcNotification> get notifications => _notificationController.stream;

  Future<Map<String, Object?>> call(
    final String method,
    final Map<String, Object?> params,
  ) async {
    await _waitForConnected();

    final String id = _newId();
    final completer = Completer<Map<String, Object?>>();
    _pending[id] = completer;

    final String payload = jsonEncode(<String, Object?>{
      'id': id,
      'method': method,
      'params': params,
    });

    _channel?.sink.add(payload);
    return completer.future.timeout(const Duration(seconds: 10));
  }

  void dispose() {
    _disposed = true;
    _subscription?.cancel();
    _subscription = null;
    _channel?.sink.close();
    _channel = null;
    for (final completer in _pending.values) {
      if (!completer.isCompleted) {
        completer.completeError(const TimeoutException('rpc_client_disposed'));
      }
    }
    _pending.clear();
    _stateController.close();
    _notificationController.close();
  }

  Future<void> _waitForConnected() async {
    if (_state == RpcConnectionState.connected) {
      return;
    }
    await for (final next in connectionStates) {
      if (next == RpcConnectionState.connected) {
        return;
      }
    }
    throw StateError('rpc_client_not_connected');
  }

  void _setState(final RpcConnectionState next) {
    _state = next;
    if (!_stateController.isClosed) {
      _stateController.add(next);
    }
  }

  Future<void> _connectLoop() async {
    if (_disposed) {
      return;
    }
    if (_channel != null) {
      return;
    }

    _setState(RpcConnectionState.connecting);
    try {
      final channel = WebSocketChannel.connect(endpoint);
      _channel = channel;
      _attempt = 0;

      _subscription = channel.stream.listen(
        _handleInbound,
        onError: (final Object error, final StackTrace st) {
          _handleDisconnect();
        },
        onDone: _handleDisconnect,
        cancelOnError: true,
      );

      _setState(RpcConnectionState.connected);
    } catch (_) {
      _handleDisconnect();
    }
  }

  void _handleDisconnect() {
    if (_disposed) {
      return;
    }

    _subscription?.cancel();
    _subscription = null;
    _channel = null;
    _setState(RpcConnectionState.disconnected);

    final int attempt = _attempt++;
    final int factor = 1 << (attempt.clamp(0, 6));
    final Duration delay = Duration(
      milliseconds: (baseBackoff.inMilliseconds * factor).clamp(
        baseBackoff.inMilliseconds,
        maxBackoff.inMilliseconds,
      ),
    );

    Future<void>.delayed(delay, _connectLoop);
  }

  void _handleInbound(final Object? message) {
    if (message is! String) {
      return;
    }

    final Object? decoded = jsonDecode(message);
    if (decoded is! Map<String, Object?>) {
      return;
    }

    final Object? idObj = decoded['id'];
    if (idObj is! String) {
      final Object? methodObj = decoded['method'];
      final Object? paramsObj = decoded['params'];
      if (methodObj is String && paramsObj is Map) {
        if (!_notificationController.isClosed) {
          _notificationController.add(
            RpcNotification(
              method: methodObj,
              params: paramsObj.cast<String, Object?>(),
            ),
          );
        }
      }
      return;
    }

    final Object? error = decoded['error'];
    final Object? result = decoded['result'];

    final completer = _pending.remove(idObj);
    if (completer == null) {
      return;
    }

    if (error != null) {
      completer.completeError(error);
      return;
    }

    if (result is Map) {
      completer.complete(result.cast<String, Object?>());
      return;
    }

    completer.complete(<String, Object?>{'value': result});
  }

  String _newId() {
    _idCounter = (_idCounter + 1) & 0x7fffffff;
    final int epoch = DateTime.now().microsecondsSinceEpoch;
    return 'rpc-$epoch-$_idCounter';
  }
}
=======
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:web_socket_channel/web_socket_channel.dart';

/// Provides the contract for local NeQST RPC communication.
abstract interface class LocalRpcClient {
  /// Fetches a JSON document from the local bridge endpoint.
  ///
  /// Intent:
  /// Retrieves typed state from the Rust bridge without exposing transport
  /// details to UI widgets.
  ///
  /// Input/Output constraints:
  /// Accepts a relative RPC path and returns a decoded JSON object map.
  ///
  /// Security implications:
  /// Callers must only pass trusted relative paths to prevent SSRF-like misuse
  /// once remote endpoints are introduced.
  Future<Map<String, Object?>> getJson(final String path);

  /// Opens a WebSocket channel for live local orchestration events.
  ///
  /// Intent:
  /// Establishes a typed bridge for state streaming from the orchestrator.
  ///
  /// Input/Output constraints:
  /// Accepts a relative path and returns a connected [WebSocketChannel].
  ///
  /// Security implications:
  /// The caller must authenticate and validate upstream messages before
  /// mutating UI state because the transport channel itself is untrusted.
  WebSocketChannel openChannel(final String path);
}

/// Implements the local RPC contract using HTTP and WebSocket transports.
final class HttpLocalRpcClient implements LocalRpcClient {
  /// Creates a local RPC client anchored to the embedded bridge base URL.
  const HttpLocalRpcClient({
    required this.baseHttpUri,
    required this.baseWebSocketUri,
    required this.httpClient,
  });

  /// Base HTTP URI of the local bridge.
  final Uri baseHttpUri;

  /// Base WebSocket URI of the local bridge.
  final Uri baseWebSocketUri;

  /// Shared HTTP client used for request reuse and testability.
  final http.Client httpClient;

  @override
  Future<Map<String, Object?>> getJson(final String path) async {
    final Uri resolvedUri = baseHttpUri.resolve(path);
    final http.Response response = await httpClient.get(resolvedUri);

    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw StateError(
        'Local RPC GET failed with ${response.statusCode} for $resolvedUri',
      );
    }

    final Object? decoded = jsonDecode(response.body);
    if (decoded is! Map<String, Object?>) {
      throw const FormatException('Local RPC response must be a JSON object.');
    }

    return decoded;
  }

  @override
  WebSocketChannel openChannel(final String path) {
    final Uri resolvedUri = baseWebSocketUri.resolve(path);
    return WebSocketChannel.connect(resolvedUri);
  }
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
