add_library(margen ${MARGEN_LIBRARY_TYPE}
	Core/Directions.cpp
	Dependency/Querying.cpp
	Graphing/Dependency/SectorsToDag.cpp
	Graphing/Dependency/SimpleDependencyGraph.cpp
	Story/GenerateStory.cpp
	Story/LocationBranching.cpp
	Structure/Story/GenerateStoryGrid.cpp
	Structure/Story/StoryToClusters.cpp
	Structure/WindingPath/GridPopulation.cpp
)

add_library(margen::margen ALIAS margen)

target_include_directories(margen
	PUBLIC
		$<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}>
		$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

target_compile_features(margen PUBLIC cxx_std_20)

target_link_libraries(margen
	PUBLIC
		mythic::mythic
)

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
	target_compile_options(margen PRIVATE -Wall -Wextra -Wpedantic)
endif()

set_target_properties(margen PROPERTIES
	EXPORT_NAME margen
	POSITION_INDEPENDENT_CODE ON
)

install(
	TARGETS margen
	EXPORT margenTargets
	ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
	LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
	RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
	INCLUDES DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)
