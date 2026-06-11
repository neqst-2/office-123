<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter/material.dart' as material;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'routing/app_sections.dart';
import 'rpc/local_rpc_client.dart';
import 'theme/app_theme.dart';
import 'workspace_tabs.dart';
import '../features/calendar/presentation/calendar_placeholder.dart';
import '../features/dashboard/presentation/dashboard_home.dart';
import '../features/documents/presentation/documents_placeholder.dart';
import '../features/mail/presentation/mail_placeholder.dart';

final StateProvider<fluent.ThemeMode> _themeModeProvider =
    StateProvider<fluent.ThemeMode>((final ref) {
  return fluent.ThemeMode.system;
});

final StateProvider<AppSection> _navSectionProvider =
    StateProvider<AppSection>((final ref) {
  return AppSection.dashboard;
});

/// Root application widget for the NeQST Office frontend shell.
class NeqstOfficeApp extends ConsumerWidget {
  /// Creates the root application widget.
  const NeqstOfficeApp({super.key});

  @override
  fluent.Widget build(
    final fluent.BuildContext context,
    final WidgetRef ref,
  ) {
    final fluent.ThemeMode themeMode = ref.watch(_themeModeProvider);
    final AppThemeBundle themeBundle = buildAppTheme(themeMode);

    return fluent.FluentApp(
      title: 'NeQST Office 1-2-3',
      debugShowCheckedModeBanner: false,
      themeMode: themeMode,
      theme: buildAppTheme(fluent.ThemeMode.light).fluentTheme,
      darkTheme: buildAppTheme(fluent.ThemeMode.dark).fluentTheme,
      home: material.Theme(
        data: themeBundle.materialTheme,
        child: const _ShellScaffold(),
      ),
    );
  }
}

/// Provides the main navigation frame with sidebar and empty tab surface.
class _ShellScaffold extends ConsumerWidget {
  /// Creates the shell scaffold.
  const _ShellScaffold();

  @override
  fluent.Widget build(
    final fluent.BuildContext context,
    final WidgetRef ref,
  ) {
    final AppSection activeSection = ref.watch(_navSectionProvider);
    final RpcConnectionState connectionState =
        ref.watch(rpcConnectionStateProvider).maybeWhen(
              data: (final s) => s,
              orElse: () => RpcConnectionState.disconnected,
            );
    final AsyncValue<NodeHealthSnapshot> nodeHealthAsync = ref.watch(nodeHealthProvider);
    final NodeHealthSnapshot? nodeHealth = nodeHealthAsync.valueOrNull;
    final bool ready = connectionState == RpcConnectionState.connected && (nodeHealth?.isReady ?? false);
    final WorkspaceTabsController tabsController =
        ref.read(workspaceTabsProvider.notifier);

    return material.AnimatedSwitcher(
      duration: const Duration(milliseconds: 350),
      child: ready
          ? fluent.NavigationView(
              key: const ValueKey<String>('workspace-ready'),
              appBar: fluent.NavigationAppBar(
                title: const fluent.Text('NeQST Office 1-2-3'),
                actions: fluent.Row(
                  mainAxisAlignment: fluent.MainAxisAlignment.end,
                  children: <fluent.Widget>[
                    fluent.ToggleSwitch(
                      checked: ref.watch(_themeModeProvider) != fluent.ThemeMode.light,
                      content: const fluent.Text('Hybrid theme'),
                      onChanged: (final bool value) {
                        ref.read(_themeModeProvider.notifier).state =
                            value ? fluent.ThemeMode.dark : fluent.ThemeMode.light;
                      },
                    ),
                    const fluent.SizedBox(width: 16),
                  ],
                ),
              ),
              pane: fluent.NavigationPane(
                selected: AppSection.values.indexOf(activeSection),
                displayMode: fluent.PaneDisplayMode.auto,
                items: <fluent.NavigationPaneItem>[
                  for (final AppSection section in AppSection.values)
                    fluent.PaneItem(
                      icon: fluent.Icon(section.iconData),
                      title: fluent.Text(section.label),
                      body: const _WorkspaceView(),
                    ),
                ],
                onChanged: (final int index) {
                  final AppSection section = AppSection.values[index];
                  ref.read(_navSectionProvider.notifier).state = section;

                  switch (section) {
                    case AppSection.dashboard:
                      tabsController.openOrActivateDashboard();
                    case AppSection.mail:
                      tabsController.openOrActivateMail();
                    case AppSection.calendar:
                      tabsController.openOrActivateCalendar();
                    case AppSection.documents:
                      tabsController.openNewDocument();
                  }
                },
              ),
            ),
          : _BootOverlay(
              key: const ValueKey<String>('workspace-boot'),
              connectionState: connectionState,
              health: nodeHealth,
            ),
    );
  }
}

class _WorkspaceView extends ConsumerWidget {
  const _WorkspaceView();

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    final RpcConnectionState connectionState =
        ref.watch(rpcConnectionStateProvider).maybeWhen(
              data: (final s) => s,
              orElse: () => RpcConnectionState.disconnected,
            );
    final WorkspaceTabsState tabsState = ref.watch(workspaceTabsProvider);
    final WorkspaceTabsController controller =
        ref.read(workspaceTabsProvider.notifier);

    final int activeIndex = tabsState.activeIndex();
    final List<WorkspaceTab> tabs = tabsState.tabs;

    return fluent.Padding(
      padding: const fluent.EdgeInsets.all(20),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          _WorkspaceHeaderBand(
            activeTab: tabs.isNotEmpty ? tabs[activeIndex.clamp(0, tabs.length - 1)] : null,
            connectionState: connectionState,
          ),
          const fluent.SizedBox(height: 20),
          fluent.Expanded(
            child: fluent.TabView(
              currentIndex: activeIndex,
              onChanged: (final int index) {
                if (index >= 0 && index < tabs.length) {
                  controller.activate(tabs[index].id);
                }
              },
              onReorder: controller.moveTab,
              onNewPressed: controller.openNewDocument,
              tabs: <fluent.Tab>[
                for (final WorkspaceTab tab in tabs)
                  fluent.Tab(
                    text: fluent.Text(tab.title),
                    icon: fluent.Icon(tab.icon),
                    body: _buildTabBody(tab),
                    onClosed: () => controller.closeTab(tab.id),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

fluent.Widget _buildTabBody(final WorkspaceTab tab) {
  final kind = tab.kind;
  return switch (kind) {
    DashboardTabKind() => const DashboardHome(),
    MailTabKind() => const MailPlaceholder(),
    CalendarTabKind() => const CalendarPlaceholder(),
    DocumentTabKind() => DocumentsPlaceholder(contextAnchor: kind.contextAnchor),
  };
}

class _WorkspaceHeaderBand extends fluent.StatelessWidget {
  const _WorkspaceHeaderBand({
    required this.activeTab,
    required this.connectionState,
  });

  final WorkspaceTab? activeTab;
  final RpcConnectionState connectionState;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.Container(
      width: double.infinity,
      padding: const fluent.EdgeInsets.symmetric(horizontal: 18, vertical: 16),
      decoration: fluent.BoxDecoration(
        color: fluent.FluentTheme.of(context).acrylicBackgroundColor,
        borderRadius: fluent.BorderRadius.circular(18),
        border: fluent.Border.all(
          color: fluent.FluentTheme.of(context).inactiveBackgroundColor,
        ),
      ),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          fluent.Text(
            activeTab?.title ?? 'Workspace',
            style: fluent.FluentTheme.of(context).typography.subtitle,
          ),
          const fluent.SizedBox(height: 6),
          fluent.Text(
            'Local RPC: ${connectionState.name}',
            style: fluent.FluentTheme.of(context).typography.caption,
          ),
        ],
      ),
    );
  }
}

class _BootOverlay extends fluent.StatelessWidget {
  const _BootOverlay({
    super.key,
    required this.connectionState,
    required this.health,
  });

  final RpcConnectionState connectionState;
  final NodeHealthSnapshot? health;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    final theme = fluent.FluentTheme.of(context);
    return fluent.Container(
      color: const fluent.Color(0xFF0E1116),
      child: fluent.Center(
        child: fluent.Container(
          constraints: const fluent.BoxConstraints(maxWidth: 640),
          padding: const fluent.EdgeInsets.all(28),
          decoration: fluent.BoxDecoration(
            color: const fluent.Color(0xFF171C24),
            borderRadius: fluent.BorderRadius.circular(24),
            border: fluent.Border.all(color: const fluent.Color(0xFF2A3340)),
          ),
          child: fluent.Column(
            mainAxisSize: fluent.MainAxisSize.min,
            crossAxisAlignment: fluent.CrossAxisAlignment.start,
            children: <fluent.Widget>[
              fluent.Text(
                'Sovereign Shield Initializing...',
                style: theme.typography.titleLarge,
              ),
              const fluent.SizedBox(height: 10),
              fluent.Text(
                'Bootstrapping secure local subsystems before unlocking the Lotus-Moderne workspace.',
                style: theme.typography.body,
              ),
              const fluent.SizedBox(height: 18),
              const fluent.ProgressRing(),
              const fluent.SizedBox(height: 18),
              _BootIndicator(
                label: 'Transport Layer',
                status: connectionState == RpcConnectionState.connected
                    ? 'connected'
                    : connectionState.name,
              ),
              _BootIndicator(
                label: 'Database',
                status: health?.dbStatus ?? 'initializing',
              ),
              _BootIndicator(
                label: 'Crypto',
                status: health?.cryptoStatus ?? 'initializing',
              ),
              _BootIndicator(
                label: 'Document Engine',
                status: health?.lokStatus == 'ready'
                    ? 'ready (${health?.lokVersion ?? 'unknown'})'
                    : (health?.lokStatus ?? 'initializing'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _BootIndicator extends fluent.StatelessWidget {
  const _BootIndicator({
    required this.label,
    required this.status,
  });

  final String label;
  final String status;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    final bool ready =
        status == 'connected' || status == 'ready' || status.startsWith('ready');
    return fluent.Padding(
      padding: const fluent.EdgeInsets.only(bottom: 8),
      child: fluent.Row(
        children: <fluent.Widget>[
          fluent.Icon(
            ready ? fluent.FluentIcons.completed : fluent.FluentIcons.sync,
            size: 16,
          ),
          const fluent.SizedBox(width: 10),
          fluent.Text('$label: $status'),
        ],
      ),
    );
  }
}
=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter/material.dart' as material;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;

import 'routing/app_sections.dart';
import 'rpc/local_rpc_client.dart';
import 'theme/app_theme.dart';
import 'workspace_tabs.dart';

final StateProvider<fluent.ThemeMode> _themeModeProvider =
    StateProvider<fluent.ThemeMode>((final ref) {
  return fluent.ThemeMode.system;
});

final Provider<LocalRpcClient> _localRpcClientProvider =
    Provider<LocalRpcClient>((final ref) {
  return HttpLocalRpcClient(
    baseHttpUri: Uri.parse('http://127.0.0.1:4312/'),
    baseWebSocketUri: Uri.parse('ws://127.0.0.1:4312/'),
    httpClient: http.Client(),
  );
});

final StateProvider<AppSection> _navSectionProvider =
    StateProvider<AppSection>((final ref) {
  return AppSection.dashboard;
});

/// Root application widget for the NeQST Office frontend shell.
class NeqstOfficeApp extends ConsumerWidget {
  /// Creates the root application widget.
  const NeqstOfficeApp({super.key});

  @override
  fluent.Widget build(
    final fluent.BuildContext context,
    final WidgetRef ref,
  ) {
    final fluent.ThemeMode themeMode = ref.watch(_themeModeProvider);
    final AppThemeBundle themeBundle = buildAppTheme(themeMode);

    return fluent.FluentApp(
      title: 'NeQST Office 1-2-3',
      debugShowCheckedModeBanner: false,
      themeMode: themeMode,
      theme: buildAppTheme(fluent.ThemeMode.light).fluentTheme,
      darkTheme: buildAppTheme(fluent.ThemeMode.dark).fluentTheme,
      home: material.Theme(
        data: themeBundle.materialTheme,
        child: const _ShellScaffold(),
      ),
    );
  }
}

/// Provides the main navigation frame with sidebar and empty tab surface.
class _ShellScaffold extends ConsumerWidget {
  /// Creates the shell scaffold.
  const _ShellScaffold();

  @override
  fluent.Widget build(
    final fluent.BuildContext context,
    final WidgetRef ref,
  ) {
    final AppSection activeSection = ref.watch(_navSectionProvider);
    final WorkspaceTabsController tabsController =
        ref.read(workspaceTabsProvider.notifier);

    return fluent.NavigationView(
      appBar: fluent.NavigationAppBar(
        title: const fluent.Text('NeQST Office 1-2-3'),
        actions: fluent.Row(
          mainAxisAlignment: fluent.MainAxisAlignment.end,
          children: <fluent.Widget>[
            fluent.ToggleSwitch(
              checked: ref.watch(_themeModeProvider) != fluent.ThemeMode.light,
              content: const fluent.Text('Hybrid theme'),
              onChanged: (final bool value) {
                ref.read(_themeModeProvider.notifier).state =
                    value ? fluent.ThemeMode.dark : fluent.ThemeMode.light;
              },
            ),
            const fluent.SizedBox(width: 16),
          ],
        ),
      ),
      pane: fluent.NavigationPane(
        selected: AppSection.values.indexOf(activeSection),
        displayMode: fluent.PaneDisplayMode.auto,
        items: <fluent.NavigationPaneItem>[
          for (final AppSection section in AppSection.values)
            fluent.PaneItem(
              icon: fluent.Icon(section.iconData),
              title: fluent.Text(section.label),
              body: const _WorkspaceView(),
            ),
        ],
        onChanged: (final int index) {
          final AppSection section = AppSection.values[index];
          ref.read(_navSectionProvider.notifier).state = section;

          switch (section) {
            case AppSection.dashboard:
              tabsController.openOrActivateDashboard();
            case AppSection.mail:
              tabsController.openOrActivateMail();
            case AppSection.calendar:
              tabsController.openOrActivateCalendar();
            case AppSection.documents:
              tabsController.openNewDocument();
          }
        },
      ),
    );
  }
}

class _WorkspaceView extends ConsumerWidget {
  const _WorkspaceView();

  @override
  fluent.Widget build(final fluent.BuildContext context, final WidgetRef ref) {
    final LocalRpcClient rpcClient = ref.watch(_localRpcClientProvider);
    final WorkspaceTabsState tabsState = ref.watch(workspaceTabsProvider);
    final WorkspaceTabsController controller =
        ref.read(workspaceTabsProvider.notifier);

    final int activeIndex = tabsState.activeIndex();
    final List<WorkspaceTab> tabs = tabsState.tabs;

    return fluent.Padding(
      padding: const fluent.EdgeInsets.all(20),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          _WorkspaceHeaderBand(
            activeTab: tabs.isNotEmpty ? tabs[activeIndex.clamp(0, tabs.length - 1)] : null,
            rpcClient: rpcClient,
          ),
          const fluent.SizedBox(height: 20),
          fluent.Expanded(
            child: fluent.TabView(
              currentIndex: activeIndex,
              onChanged: (final int index) {
                if (index >= 0 && index < tabs.length) {
                  controller.activate(tabs[index].id);
                }
              },
              onReorder: controller.moveTab,
              onNewPressed: controller.openNewDocument,
              tabs: <fluent.Tab>[
                for (final WorkspaceTab tab in tabs)
                  fluent.Tab(
                    text: fluent.Text(tab.title),
                    icon: fluent.Icon(tab.icon),
                    body: tab.content.build(),
                    onClosed: () => controller.closeTab(tab.id),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _WorkspaceHeaderBand extends fluent.StatelessWidget {
  const _WorkspaceHeaderBand({
    required this.activeTab,
    required this.rpcClient,
  });

  final WorkspaceTab? activeTab;
  final LocalRpcClient rpcClient;

  @override
  fluent.Widget build(final fluent.BuildContext context) {
    return fluent.Container(
      width: double.infinity,
      padding: const fluent.EdgeInsets.symmetric(horizontal: 18, vertical: 16),
      decoration: fluent.BoxDecoration(
        color: fluent.FluentTheme.of(context).acrylicBackgroundColor,
        borderRadius: fluent.BorderRadius.circular(18),
        border: fluent.Border.all(
          color: fluent.FluentTheme.of(context).inactiveBackgroundColor,
        ),
      ),
      child: fluent.Column(
        crossAxisAlignment: fluent.CrossAxisAlignment.start,
        children: <fluent.Widget>[
          fluent.Text(
            activeTab?.title ?? 'Workspace',
            style: fluent.FluentTheme.of(context).typography.subtitle,
          ),
          const fluent.SizedBox(height: 6),
          fluent.Text(
            'Local RPC: ${rpcClient.runtimeType}',
            style: fluent.FluentTheme.of(context).typography.caption,
          ),
        ],
      ),
    );
  }
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
