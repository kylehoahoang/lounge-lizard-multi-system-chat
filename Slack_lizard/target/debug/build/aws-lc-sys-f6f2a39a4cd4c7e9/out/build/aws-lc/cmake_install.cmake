# Install script for directory: C:/Users/sorry/.cargo/registry/src/index.crates.io-6f17d22bba15001f/aws-lc-sys-0.21.1/aws-lc

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "C:/Users/sorry/Desktop/Discord Engineering/design2_discord_kyle_branch/Slack_lizard/target/debug/build/aws-lc-sys-f6f2a39a4cd4c7e9/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Release")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Development" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE DIRECTORY FILES "C:/Users/sorry/.cargo/registry/src/index.crates.io-6f17d22bba15001f/aws-lc-sys-0.21.1/aws-lc/include/openssl" REGEX "/boringssl\\_prefix\\_symbols\\.h$" EXCLUDE REGEX "/boringssl\\_prefix\\_symbols\\_asm\\.h$" EXCLUDE REGEX "/boringssl\\_prefix\\_symbols\\_nasm\\.inc$" EXCLUDE)
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Development" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE DIRECTORY FILES "C:/Users/sorry/Desktop/Discord Engineering/design2_discord_kyle_branch/Slack_lizard/target/debug/build/aws-lc-sys-f6f2a39a4cd4c7e9/out/build/aws-lc/symbol_prefix_include/openssl")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for each subdirectory.
  include("C:/Users/sorry/Desktop/Discord Engineering/design2_discord_kyle_branch/Slack_lizard/target/debug/build/aws-lc-sys-f6f2a39a4cd4c7e9/out/build/aws-lc/crypto/cmake_install.cmake")
  include("C:/Users/sorry/Desktop/Discord Engineering/design2_discord_kyle_branch/Slack_lizard/target/debug/build/aws-lc-sys-f6f2a39a4cd4c7e9/out/build/aws-lc/util/fipstools/cmake_install.cmake")

endif()

