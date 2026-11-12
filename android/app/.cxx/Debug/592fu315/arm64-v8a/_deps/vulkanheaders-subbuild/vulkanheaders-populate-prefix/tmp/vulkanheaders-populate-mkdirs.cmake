# Distributed under the OSI-approved BSD 3-Clause License.  See accompanying
# file LICENSE.rst or https://cmake.org/licensing for details.

cmake_minimum_required(VERSION ${CMAKE_VERSION}) # this file comes with cmake

# If CMAKE_DISABLE_SOURCE_CHANGES is set to true and the source directory is an
# existing directory in our source tree, calling file(MAKE_DIRECTORY) on it
# would cause a fatal error, even though it would be a no-op.
if(NOT EXISTS "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-src")
  file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-src")
endif()
file(MAKE_DIRECTORY
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-build"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/tmp"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/src/vulkanheaders-populate-stamp"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/src"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/src/vulkanheaders-populate-stamp"
)

set(configSubDirs )
foreach(subDir IN LISTS configSubDirs)
    file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/src/vulkanheaders-populate-stamp/${subDir}")
endforeach()
if(cfgdir)
  file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/592fu315/arm64-v8a/_deps/vulkanheaders-subbuild/vulkanheaders-populate-prefix/src/vulkanheaders-populate-stamp${cfgdir}") # cfgdir has leading slash
endif()
