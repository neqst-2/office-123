<<<<<<< HEAD
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/app_shell.dart';

/// Starts the NeQST Office frontend shell.
///
/// Intent:
/// Creates the root provider scope and boots the cross-platform UI.
///
/// Input/Output constraints:
/// Takes no runtime arguments and returns no value.
///
/// Security implications:
/// This entry point must remain side-effect light and must not initialize
/// direct filesystem, database, or raw network access without passing through
/// explicitly audited service abstractions.
void main() {
  runApp(const ProviderScope(child: NeqstOfficeApp()));
}
=======
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'core/app_shell.dart';

/// Starts the NeQST Office frontend shell.
///
/// Intent:
/// Creates the root provider scope and boots the cross-platform UI.
///
/// Input/Output constraints:
/// Takes no runtime arguments and returns no value.
///
/// Security implications:
/// This entry point must remain side-effect light and must not initialize
/// direct filesystem, database, or raw network access without passing through
/// explicitly audited service abstractions.
void main() {
  runApp(const ProviderScope(child: NeqstOfficeApp()));
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
