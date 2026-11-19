plugins {
    id("com.android.application")
    // AGP 9 has built-in Kotlin support; only the Compose compiler plugin is applied.
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.example.tokenusage"
    compileSdk = 37

    defaultConfig {
        // A new applicationId lets this build install alongside older releases
        // whose runner-generated debug signatures cannot be updated in place.
        applicationId = "com.example.tokenusage.live"
        minSdk = 26
        targetSdk = 37
        versionCode = 3
        versionName = "0.3.3"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    // A stable, committed debug keystore means every CI run produces an APK
    // with the same signature, so users can update installed builds.
    signingConfigs {
        getByName("debug") {
            storeFile = file("keystore/debug.jks")
            storePassword = "android"
            keyAlias = "androiddebugkey"
            keyPassword = "android"
        }
    }

    buildTypes {
        getByName("debug") {
            signingConfig = signingConfigs.getByName("debug")
        }
        getByName("release") {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        // Built-in Kotlin derives jvmTarget from targetCompatibility.
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildFeatures {
        compose = true
    }

    testOptions {
        unitTests {
            // Required by the Compose Preview task in AGP 9.
            isIncludeAndroidResources = true
        }
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2026.09.00")
    implementation(composeBom)
    androidTestImplementation(composeBom)

    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.navigation:navigation-compose:2.10.2")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.11.0")
    implementation("androidx.datastore:datastore-preferences:1.3.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.11.0")

    testImplementation("junit:junit:4.13.2")
}
