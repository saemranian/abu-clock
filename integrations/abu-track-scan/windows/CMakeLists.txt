cmake_minimum_required(VERSION 3.22)
project(AbuClockWinClient LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(AbuClockWinClient STATIC
    AbuClockWinClient.cpp
    AbuClockWinClient.hpp
)

target_include_directories(AbuClockWinClient PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../crates/abu-ffi/include
)
