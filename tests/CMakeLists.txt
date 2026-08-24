cmake_minimum_required(VERSION 3.21)

include(FetchContent)

FetchContent_Declare(
	Catch2
	GIT_REPOSITORY https://github.com/catchorg/Catch2.git
	GIT_TAG v3.5.4
)

FetchContent_MakeAvailable(Catch2)

list(APPEND CMAKE_MODULE_PATH ${Catch2_SOURCE_DIR}/extras)

add_executable(mythic_tests
	mythic/DiceTest.cpp
	mythic/ContainersTest.cpp
	mythic/Vector3Test.cpp
)

target_link_libraries(mythic_tests
	PRIVATE
		mythic::mythic
		Catch2::Catch2WithMain
)

add_executable(margen_tests
	margen/Core/DirectionsTest.cpp
	margen/Story/GenerateStoryTest.cpp
	margen/Story/LocationBranchingTest.cpp
	margen/Dependency/QueryingTest.cpp
	margen/Graphing/Dependency/SectorsToDagTest.cpp
	margen/Structure/StoryToClustersTest.cpp
)

target_link_libraries(margen_tests
	PRIVATE
		margen::margen
		Catch2::Catch2WithMain
)

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
	target_compile_options(mythic_tests PRIVATE -Wall -Wextra -Wpedantic)
	target_compile_options(margen_tests PRIVATE -Wall -Wextra -Wpedantic)
endif()

include(Catch)
catch_discover_tests(mythic_tests)
catch_discover_tests(margen_tests)
