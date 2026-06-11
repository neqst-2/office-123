<<<<<<< HEAD
cmake_minimum_required(VERSION 3.14)

project(neqst_office_123_windows_runner LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(neqst_office_123 WIN32
  main.cpp
)

set(FLUTTER_MANAGED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter")
set(FLUTTER_EPHEMERAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter/ephemeral")

target_include_directories(neqst_office_123 PRIVATE
  "${FLUTTER_EPHEMERAL_DIR}"
  "${FLUTTER_EPHEMERAL_DIR}/cpp_client_wrapper/include"
)

target_link_directories(neqst_office_123 PRIVATE
  "${FLUTTER_EPHEMERAL_DIR}"
)

target_link_libraries(neqst_office_123 PRIVATE
  flutter
  flutter_wrapper_app
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${FLUTTER_EPHEMERAL_DIR}/flutter_windows.dll"
          $<TARGET_FILE_DIR:neqst_office_123>/flutter_windows.dll
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_directory
          "${CMAKE_CURRENT_SOURCE_DIR}/data"
          $<TARGET_FILE_DIR:neqst_office_123>/data
)

=======
cmake_minimum_required(VERSION 3.14)

project(neqst_office_123_windows_runner LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(neqst_office_123 WIN32
  main.cpp
)

set(FLUTTER_MANAGED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter")
set(FLUTTER_EPHEMERAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/flutter/ephemeral")

target_include_directories(neqst_office_123 PRIVATE
  "${FLUTTER_EPHEMERAL_DIR}"
  "${FLUTTER_EPHEMERAL_DIR}/cpp_client_wrapper/include"
)

target_link_directories(neqst_office_123 PRIVATE
  "${FLUTTER_EPHEMERAL_DIR}"
)

target_link_libraries(neqst_office_123 PRIVATE
  flutter
  flutter_wrapper_app
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          "${FLUTTER_EPHEMERAL_DIR}/flutter_windows.dll"
          $<TARGET_FILE_DIR:neqst_office_123>/flutter_windows.dll
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_directory
          "${CMAKE_CURRENT_SOURCE_DIR}/data"
          $<TARGET_FILE_DIR:neqst_office_123>/data
)

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
