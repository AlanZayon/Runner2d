plugins {
    id("com.android.application")
}

val vsVersionCode = 16
val vsVersionName = "0.16.0"

android {
    namespace = "com.verticalshift.game"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.verticalshift.game"
        minSdk = 26
        targetSdk = 34
        versionCode = vsVersionCode
        versionName = vsVersionName
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
        externalNativeBuild {
            cmake {
                arguments += listOf(
                    "-DANDROID_STL=c++_shared",
                    "-DVS_BUILD_TESTS=OFF",
                    "-DVS_APP_VERSION=$vsVersionName",
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
