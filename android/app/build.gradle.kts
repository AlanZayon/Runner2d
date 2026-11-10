plugins {
    id("com.android.application")
}

android {
    namespace = "com.verticalshift.game"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.verticalshift.game"
        minSdk = 26
        targetSdk = 34
        versionCode = 12
        versionName = "0.12.0-phase12"
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
        externalNativeBuild {
            cmake {
                arguments += listOf(
                    "-DANDROID_STL=c++_shared",
                    "-DVS_BUILD_TESTS=OFF",
                )
            }
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
        }
        getByName("debug") {
            isDebuggable = true
        }
    }

    externalNativeBuild {
        cmake {
            path = file("../../CMakeLists.txt")
            version = "3.24.0+"
        }
    }

    sourceSets {
        getByName("main") {
            assets.srcDirs(
                "../../assets",
                "../../build/android-assets",
            )
        }
    }

    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

dependencies {
    // Game Activity available for future Java shell swap; native uses NDK glue today.
    implementation("androidx.games:games-activity:3.0.5")
}
