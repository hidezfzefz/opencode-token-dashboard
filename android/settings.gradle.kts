pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

plugins {
    // Android Gradle Plugin 9.x supports Gradle 9 (this project uses Gradle 9.7)
    // and provides built-in Kotlin support, so no `kotlin-android` plugin is needed.
    id("com.android.application") version "9.5.1" apply false
    // Compose compiler plugin (versioned in lockstep with Kotlin 2.x).
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.21" apply false
}

rootProject.name = "TokenUsage"
