cmake_minimum_required(VERSION 3.25)

project(query C)

set(CMAKE_C_STANDARD, 99)

include_directories(.)
link_directories(../target/release/)

add_executable(query query.c)
target_link_libraries(query libqueryer_c.dylib)
