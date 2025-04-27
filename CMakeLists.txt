cmake_minimum_required(VERSION 3.10)

# Make sure this matches ./NAM/version.h!
project(NAM VERSION 0.3.0)

set(CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED OFF)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++20")

if (CMAKE_SYSTEM_NAME STREQUAL "Darwin")
	include_directories(SYSTEM /usr/local/include)
elseif (CMAKE_SYSTEM_NAME STREQUAL "Linux")
	link_libraries(stdc++fs)
elseif (CMAKE_SYSTEM_NAME STREQUAL "Windows")
	add_compile_definitions(NOMINMAX WIN32_LEAN_AND_MEAN)
else()
	message(FATAL_ERROR "Unrecognized Platform!")
endif()

set(NAM_DEPS_PATH "${CMAKE_CURRENT_SOURCE_DIR}/Dependencies")

include_directories(${NAM_DEPS_PATH}/eigen)
include_directories(${NAM_DEPS_PATH}/nlohmann)


file(GLOB_RECURSE NAM_SOURCES NAM/*.cpp NAM/*.c NAM/*.h)
add_library(nam_core_static STATIC ${NAM_SOURCES}) 