# Distributed under the OSI-approved BSD 3-Clause License.  See accompanying
# file LICENSE.rst or https://cmake.org/licensing for details.

cmake_minimum_required(VERSION ${CMAKE_VERSION}) # this file comes with cmake

# If CMAKE_DISABLE_SOURCE_CHANGES is set to true and the source directory is an
# existing directory in our source tree, calling file(MAKE_DIRECTORY) on it
# would cause a fatal error, even though it would be a no-op.
if(NOT EXISTS "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-src")
  file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-src")
endif()
file(MAKE_DIRECTORY
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-build"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/tmp"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/src/volk-populate-stamp"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/src"
  "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/src/volk-populate-stamp"
)

set(configSubDirs )
foreach(subDir IN LISTS configSubDirs)
    file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/src/volk-populate-stamp/${subDir}")
endforeach()
if(cfgdir)
  file(MAKE_DIRECTORY "C:/Users/alanz/source/repos/Runner2d/android/app/.cxx/Debug/535k1s2r/arm64-v8a/_deps/volk-subbuild/volk-populate-prefix/src/volk-populate-stamp${cfgdir}") # cfgdir has leading slash
endif()
