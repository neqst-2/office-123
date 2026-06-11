<<<<<<< HEAD
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

sealed class WorkspaceTabKind {
  const WorkspaceTabKind();
}

final class DashboardTabKind extends WorkspaceTabKind {
  const DashboardTabKind();
}

final class MailTabKind extends WorkspaceTabKind {
  const MailTabKind();
}

final class CalendarTabKind extends WorkspaceTabKind {
  const CalendarTabKind();
}

final class DocumentTabKind extends WorkspaceTabKind {
  const DocumentTabKind({this.contextAnchor});

  final String? contextAnchor;
}

final class WorkspaceTab {
  const WorkspaceTab({
    required this.id,
    required this.title,
    required this.icon,
    required this.kind,
    this.isPinned = false,
  });

  final String id;
  final String title;
  final fluent.IconData icon;
  final WorkspaceTabKind kind;
  final bool isPinned;

  WorkspaceTab copyWith({
    final String? id,
    final String? title,
    final fluent.IconData? icon,
    final WorkspaceTabKind? kind,
    final bool? isPinned,
  }) {
    return WorkspaceTab(
      id: id ?? this.id,
      title: title ?? this.title,
      icon: icon ?? this.icon,
      kind: kind ?? this.kind,
      isPinned: isPinned ?? this.isPinned,
    );
  }
}

final class WorkspaceTabsState {
  const WorkspaceTabsState({
    required this.tabs,
    required this.activeTabId,
  });

  final List<WorkspaceTab> tabs;
  final String activeTabId;

  int activeIndex() {
    final int index = tabs.indexWhere((final t) => t.id == activeTabId);
    return index >= 0 ? index : 0;
  }

  WorkspaceTabsState copyWith({
    final List<WorkspaceTab>? tabs,
    final String? activeTabId,
  }) {
    return WorkspaceTabsState(
      tabs: tabs ?? this.tabs,
      activeTabId: activeTabId ?? this.activeTabId,
    );
  }
}

final workspaceTabsProvider =
    StateNotifierProvider<WorkspaceTabsController, WorkspaceTabsState>(
  (final ref) => WorkspaceTabsController.initial(),
);

final class WorkspaceTabsController extends StateNotifier<WorkspaceTabsState> {
  WorkspaceTabsController(final WorkspaceTabsState state) : super(state);

  factory WorkspaceTabsController.initial() {
    const WorkspaceTab dashboard = WorkspaceTab(
      id: 'dashboard',
      title: 'Dashboard',
      icon: fluent.FluentIcons.home,
      kind: DashboardTabKind(),
      isPinned: true,
    );

    return WorkspaceTabsController(
      const WorkspaceTabsState(tabs: <WorkspaceTab>[dashboard], activeTabId: 'dashboard'),
    );
  }

  void activate(final String tabId) {
    if (state.tabs.any((final t) => t.id == tabId)) {
      state = state.copyWith(activeTabId: tabId);
    }
  }

  void openTab(final WorkspaceTab tab) {
    final int existingIndex = state.tabs.indexWhere((final t) => t.id == tab.id);
    if (existingIndex >= 0) {
      state = state.copyWith(activeTabId: tab.id);
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs, tab];
    state = state.copyWith(tabs: nextTabs, activeTabId: tab.id);
  }

  void closeTab(final String tabId) {
    final int index = state.tabs.indexWhere((final t) => t.id == tabId);
    if (index < 0) {
      return;
    }

    final WorkspaceTab tab = state.tabs[index];
    if (tab.isPinned) {
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs]..removeAt(index);
    if (nextTabs.isEmpty) {
      return;
    }

    final String nextActiveId = state.activeTabId == tabId
        ? nextTabs[(index - 1).clamp(0, nextTabs.length - 1)].id
        : state.activeTabId;

    state = state.copyWith(tabs: nextTabs, activeTabId: nextActiveId);
  }

  void moveTab(final int oldIndex, final int newIndex) {
    if (oldIndex < 0 || oldIndex >= state.tabs.length) {
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs];
    final WorkspaceTab tab = nextTabs.removeAt(oldIndex);

    final int insertIndex = oldIndex < newIndex ? newIndex - 1 : newIndex;
    nextTabs.insert(insertIndex.clamp(0, nextTabs.length), tab);

    state = state.copyWith(tabs: nextTabs);
  }

  void openOrActivateMail() {
    const String id = 'mail';
    if (state.tabs.any((final t) => t.id == id)) {
      activate(id);
      return;
    }

    openTab(
      const WorkspaceTab(
        id: id,
        title: 'Mail',
        icon: fluent.FluentIcons.mail,
        kind: MailTabKind(),
        isPinned: true,
      ),
    );
  }

  void openOrActivateCalendar() {
    const String id = 'calendar';
    if (state.tabs.any((final t) => t.id == id)) {
      activate(id);
      return;
    }

    openTab(
      const WorkspaceTab(
        id: id,
        title: 'Calendar',
        icon: fluent.FluentIcons.calendar,
        kind: CalendarTabKind(),
        isPinned: true,
      ),
    );
  }

  void openOrActivateDashboard() => activate('dashboard');

  void openLinkedSpreadsheetFromMail({
    required final String contextAnchor,
  }) {
    final String id = 'doc-${DateTime.now().microsecondsSinceEpoch}';
    openTab(
      WorkspaceTab(
        id: id,
        title: 'Linked Spreadsheet',
        icon: fluent.FluentIcons.excel_document,
        kind: DocumentTabKind(contextAnchor: contextAnchor),
      ),
    );
  }

  void openNewDocument() {
    final String id = 'doc-${DateTime.now().microsecondsSinceEpoch}';
    openTab(
      WorkspaceTab(
        id: id,
        title: 'New Document',
        icon: fluent.FluentIcons.document,
        kind: const DocumentTabKind(),
      ),
    );
  }
}

=======
import 'package:fluent_ui/fluent_ui.dart' as fluent;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../features/calendar/presentation/calendar_placeholder.dart';
import '../features/dashboard/presentation/dashboard_home.dart';
import '../features/documents/presentation/documents_placeholder.dart';
import '../features/mail/presentation/mail_placeholder.dart';

sealed class WorkspaceTabContent {
  const WorkspaceTabContent();

  fluent.Widget build();
}

final class DashboardTabContent extends WorkspaceTabContent {
  const DashboardTabContent();

  @override
  fluent.Widget build() => const DashboardHome();
}

final class MailTabContent extends WorkspaceTabContent {
  const MailTabContent();

  @override
  fluent.Widget build() => const MailPlaceholder();
}

final class CalendarTabContent extends WorkspaceTabContent {
  const CalendarTabContent();

  @override
  fluent.Widget build() => const CalendarPlaceholder();
}

final class DocumentTabContent extends WorkspaceTabContent {
  const DocumentTabContent({this.contextAnchor});

  final String? contextAnchor;

  @override
  fluent.Widget build() => DocumentsPlaceholder(contextAnchor: contextAnchor);
}

final class WorkspaceTab {
  const WorkspaceTab({
    required this.id,
    required this.title,
    required this.icon,
    required this.content,
    this.isPinned = false,
  });

  final String id;
  final String title;
  final fluent.IconData icon;
  final WorkspaceTabContent content;
  final bool isPinned;

  WorkspaceTab copyWith({
    final String? id,
    final String? title,
    final fluent.IconData? icon,
    final WorkspaceTabContent? content,
    final bool? isPinned,
  }) {
    return WorkspaceTab(
      id: id ?? this.id,
      title: title ?? this.title,
      icon: icon ?? this.icon,
      content: content ?? this.content,
      isPinned: isPinned ?? this.isPinned,
    );
  }
}

final class WorkspaceTabsState {
  const WorkspaceTabsState({
    required this.tabs,
    required this.activeTabId,
  });

  final List<WorkspaceTab> tabs;
  final String activeTabId;

  int activeIndex() {
    final int index = tabs.indexWhere((final t) => t.id == activeTabId);
    return index >= 0 ? index : 0;
  }

  WorkspaceTabsState copyWith({
    final List<WorkspaceTab>? tabs,
    final String? activeTabId,
  }) {
    return WorkspaceTabsState(
      tabs: tabs ?? this.tabs,
      activeTabId: activeTabId ?? this.activeTabId,
    );
  }
}

final workspaceTabsProvider =
    StateNotifierProvider<WorkspaceTabsController, WorkspaceTabsState>(
  (final ref) => WorkspaceTabsController.initial(),
);

final class WorkspaceTabsController extends StateNotifier<WorkspaceTabsState> {
  WorkspaceTabsController(final WorkspaceTabsState state) : super(state);

  factory WorkspaceTabsController.initial() {
    const WorkspaceTab dashboard = WorkspaceTab(
      id: 'dashboard',
      title: 'Dashboard',
      icon: fluent.FluentIcons.home,
      content: DashboardTabContent(),
      isPinned: true,
    );

    return WorkspaceTabsController(
      const WorkspaceTabsState(tabs: <WorkspaceTab>[dashboard], activeTabId: 'dashboard'),
    );
  }

  void activate(final String tabId) {
    if (state.tabs.any((final t) => t.id == tabId)) {
      state = state.copyWith(activeTabId: tabId);
    }
  }

  void openTab(final WorkspaceTab tab) {
    final int existingIndex = state.tabs.indexWhere((final t) => t.id == tab.id);
    if (existingIndex >= 0) {
      state = state.copyWith(activeTabId: tab.id);
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs, tab];
    state = state.copyWith(tabs: nextTabs, activeTabId: tab.id);
  }

  void closeTab(final String tabId) {
    final int index = state.tabs.indexWhere((final t) => t.id == tabId);
    if (index < 0) {
      return;
    }

    final WorkspaceTab tab = state.tabs[index];
    if (tab.isPinned) {
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs]..removeAt(index);
    if (nextTabs.isEmpty) {
      return;
    }

    final String nextActiveId = state.activeTabId == tabId
        ? nextTabs[(index - 1).clamp(0, nextTabs.length - 1)].id
        : state.activeTabId;

    state = state.copyWith(tabs: nextTabs, activeTabId: nextActiveId);
  }

  void moveTab(final int oldIndex, final int newIndex) {
    if (oldIndex < 0 || oldIndex >= state.tabs.length) {
      return;
    }

    final List<WorkspaceTab> nextTabs = <WorkspaceTab>[...state.tabs];
    final WorkspaceTab tab = nextTabs.removeAt(oldIndex);

    final int insertIndex = oldIndex < newIndex ? newIndex - 1 : newIndex;
    nextTabs.insert(insertIndex.clamp(0, nextTabs.length), tab);

    state = state.copyWith(tabs: nextTabs);
  }

  void openOrActivateMail() {
    const String id = 'mail';
    if (state.tabs.any((final t) => t.id == id)) {
      activate(id);
      return;
    }

    openTab(
      const WorkspaceTab(
        id: id,
        title: 'Mail',
        icon: fluent.FluentIcons.mail,
        content: MailTabContent(),
        isPinned: true,
      ),
    );
  }

  void openOrActivateCalendar() {
    const String id = 'calendar';
    if (state.tabs.any((final t) => t.id == id)) {
      activate(id);
      return;
    }

    openTab(
      const WorkspaceTab(
        id: id,
        title: 'Calendar',
        icon: fluent.FluentIcons.calendar,
        content: CalendarTabContent(),
        isPinned: true,
      ),
    );
  }

  void openOrActivateDashboard() => activate('dashboard');

  void openLinkedSpreadsheetFromMail({
    required final String contextAnchor,
  }) {
    final String id = 'doc-${DateTime.now().microsecondsSinceEpoch}';
    openTab(
      WorkspaceTab(
        id: id,
        title: 'Linked Spreadsheet',
        icon: fluent.FluentIcons.excel_document,
        content: DocumentTabContent(contextAnchor: contextAnchor),
      ),
    );
  }

  void openNewDocument() {
    final String id = 'doc-${DateTime.now().microsecondsSinceEpoch}';
    openTab(
      WorkspaceTab(
        id: id,
        title: 'New Document',
        icon: fluent.FluentIcons.document,
        content: const DocumentTabContent(),
      ),
    );
  }
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
