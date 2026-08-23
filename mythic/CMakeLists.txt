add_library(mythic ${MARGEN_LIBRARY_TYPE}
	Mythic/Dice.cpp
)

add_library(mythic::mythic ALIAS mythic)

target_include_directories(mythic
	PUBLIC
		$<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}>
		$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

target_compile_features(mythic PUBLIC cxx_std_20)

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
	target_compile_options(mythic PRIVATE -Wall -Wextra -Wpedantic)
endif()

set_target_properties(mythic PROPERTIES
	EXPORT_NAME mythic
	POSITION_INDEPENDENT_CODE ON
)

install(
	TARGETS mythic
	EXPORT mythicTargets
	ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
	LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
	RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
	INCLUDES DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)
