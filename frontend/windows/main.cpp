<<<<<<< HEAD
#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <flutter/window_proc_delegate.h>

#include <windows.h>

int APIENTRY wWinMain(_In_ HINSTANCE instance,
                      _In_opt_ HINSTANCE prev,
                      _In_ wchar_t* command_line,
                      _In_ int show_command) {
  flutter::DartProject project(L"data");
  flutter::FlutterViewController controller(1280, 800, project);
  HWND window = controller.GetNativeWindow();

  if (!window) {
    return EXIT_FAILURE;
  }

  ShowWindow(window, show_command);
  UpdateWindow(window);

  MSG msg;
  while (GetMessage(&msg, nullptr, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
  }

  return EXIT_SUCCESS;
}

=======
#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <flutter/window_proc_delegate.h>

#include <windows.h>

int APIENTRY wWinMain(_In_ HINSTANCE instance,
                      _In_opt_ HINSTANCE prev,
                      _In_ wchar_t* command_line,
                      _In_ int show_command) {
  flutter::DartProject project(L"data");
  flutter::FlutterViewController controller(1280, 800, project);
  HWND window = controller.GetNativeWindow();

  if (!window) {
    return EXIT_FAILURE;
  }

  ShowWindow(window, show_command);
  UpdateWindow(window);

  MSG msg;
  while (GetMessage(&msg, nullptr, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
  }

  return EXIT_SUCCESS;
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
