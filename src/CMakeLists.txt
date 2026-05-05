cmake_minimum_required(VERSION 3.10)

# 1. Add CXX (C++) to the supported languages
project(audiopc_library VERSION 0.0.1 LANGUAGES C)

add_library(audiopc SHARED
  "audiopc.c"
)

set_target_properties(audiopc PROPERTIES
  PUBLIC_HEADER audiopc.h
  OUTPUT_NAME "audiopc"
)

target_compile_definitions(audiopc PUBLIC DART_SHARED_LIB)

if (ANDROID)
  # Support Android 15 16k page size
  target_link_options(audiopc PRIVATE "-Wl,-z,max-page-size=16384")
endif()