<<<<<<< HEAD
cmake_minimum_required(VERSION 3.14)

project(neqst_office_123_linux_runner LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK REQUIRED gtk+-3.0)

add_executable(neqst_office_123
  main.cc
)

target_include_directories(neqst_office_123 PRIVATE
  ${GTK_INCLUDE_DIRS}
  "${CMAKE_CURRENT_SOURCE_DIR}/flutter/ephemeral"
)

target_compile_options(neqst_office_123 PRIVATE ${GTK_CFLAGS_OTHER})
target_link_libraries(neqst_office_123 PRIVATE ${GTK_LIBRARIES})
target_link_directories(neqst_office_123 PRIVATE ${GTK_LIBRARY_DIRS})

target_link_libraries(neqst_office_123 PRIVATE
  flutter_linux
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_directory
          "${CMAKE_CURRENT_SOURCE_DIR}/data"
          $<TARGET_FILE_DIR:neqst_office_123>/data
)

=======
cmake_minimum_required(VERSION 3.14)

project(neqst_office_123_linux_runner LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK REQUIRED gtk+-3.0)

add_executable(neqst_office_123
  main.cc
)

target_include_directories(neqst_office_123 PRIVATE
  ${GTK_INCLUDE_DIRS}
  "${CMAKE_CURRENT_SOURCE_DIR}/flutter/ephemeral"
)

target_compile_options(neqst_office_123 PRIVATE ${GTK_CFLAGS_OTHER})
target_link_libraries(neqst_office_123 PRIVATE ${GTK_LIBRARIES})
target_link_directories(neqst_office_123 PRIVATE ${GTK_LIBRARY_DIRS})

target_link_libraries(neqst_office_123 PRIVATE
  flutter_linux
)

add_custom_command(TARGET neqst_office_123 POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_directory
          "${CMAKE_CURRENT_SOURCE_DIR}/data"
          $<TARGET_FILE_DIR:neqst_office_123>/data
)

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
