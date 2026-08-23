cmake_minimum_required(VERSION 3.21)

project(margen
    VERSION 0.1.0
    DESCRIPTION "Engine-agnostic maze-like world generation"
    LANGUAGES CXX
)

option(MARGEN_BUILD_TESTING "Build margen unit tests" ON)
option(MARGEN_BUILD_SHARED "Build shared libraries instead of static" OFF)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

if(MARGEN_BUILD_SHARED)
    set(MARGEN_LIBRARY_TYPE SHARED)
else()
    set(MARGEN_LIBRARY_TYPE STATIC)
endif()

add_subdirectory(mythic)
add_subdirectory(margen)

if(MARGEN_BUILD_TESTING)
    include(CTest)
    if(BUILD_TESTING)
        add_subdirectory(tests)
    endif()
endif()

# Install headers colocated with sources (*.h only; no separate include/ tree)
install(
    DIRECTORY mythic margen
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    FILES_MATCHING
        PATTERN "*.h"
        PATTERN "CMakeLists.txt" EXCLUDE
)

install(
    EXPORT margenTargets
    FILE margenTargets.cmake
    NAMESPACE margen::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/margen
)

# Also export mythic under mythic:: for consumers that only need utilities.
install(
    EXPORT mythicTargets
    FILE mythicTargets.cmake
    NAMESPACE mythic::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mythic
)

configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/cmake/margenConfig.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/margenConfig.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/margen
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/margenConfigVersion.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/cmake/mythicConfig.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/mythicConfig.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mythic
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/mythicConfigVersion.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(
    FILES
        ${CMAKE_CURRENT_BINARY_DIR}/margenConfig.cmake
        ${CMAKE_CURRENT_BINARY_DIR}/margenConfigVersion.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/margen
)

install(
    FILES
        ${CMAKE_CURRENT_BINARY_DIR}/mythicConfig.cmake
        ${CMAKE_CURRENT_BINARY_DIR}/mythicConfigVersion.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mythic
)
