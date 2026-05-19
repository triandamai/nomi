import java.io.FileInputStream
import java.util.*
import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
    alias(libs.plugins.kotlinXSerialization)
    alias(libs.plugins.androidx.room)
    alias(libs.plugins.ksp)
}

kotlin {
    @OptIn(ExperimentalKotlinGradlePluginApi::class)
    compilerOptions {
        //        jvmTarget.set(JvmTarget.JVM_11)
    }

    androidTarget {
        @OptIn(ExperimentalKotlinGradlePluginApi::class)
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
            freeCompilerArgs.addAll(
                "-Xsuppress-warning=EXPECT_ACTUAL_CLASSIFIERS_ARE_IN_BETA_WARNING"
            )
            optIn.addAll(
                "kotlin.js.ExperimentalJsExport",
            )
        }
    }

    listOf(iosX64(), iosArm64(), iosSimulatorArm64()).forEach { iosTarget ->
        iosTarget.binaries.framework {
            baseName = "ComposeApp"
            isStatic = true
            // Required when using NativeSQLiteDriver
            linkerOpts.add("-lsqlite3")
        }
    }

    jvm {
        @OptIn(ExperimentalKotlinGradlePluginApi::class)
        compilerOptions { jvmTarget.set(JvmTarget.JVM_11) }
    }

    sourceSets {
        configurations.all { resolutionStrategy { force(libs.kotlinx.datetime) } }
        androidMain.dependencies {
            implementation(libs.ui.tooling.preview)
            implementation(libs.androidx.core.ktx)
            implementation(libs.androidx.activity.compose)
            implementation(libs.ktor.client.okhttp)
            implementation(libs.ktor.client.android)
            implementation(libs.koin.android)
            implementation(libs.androidx.multidex)
            implementation(libs.androidx.room.sqlite.wrapper)

            implementation(project.dependencies.platform(libs.firebase.bom))
            implementation(libs.firebase.analytics)
            implementation(libs.firebase.messaging)
        }
        commonMain.dependencies {
            implementation(libs.runtime)
            implementation(libs.foundation)
            implementation(libs.material3)
            implementation(libs.ui)
            implementation(libs.components.resources)
            implementation(libs.ui.tooling.preview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)
            implementation(libs.compose.webview)

            implementation("com.composables:icons-lucide-cmp:2.2.1")
            implementation(libs.jetbrains.navigation3.ui)

            implementation(libs.shimmer)

            implementation(libs.ktor.client.core)
            implementation(libs.ktor.client.content.negotiation)
            implementation(libs.ktor.client.serialization.kotlinx.json)

            implementation(libs.kotlinx.datetime)

            implementation(libs.composeIcons.feather)

            implementation(project.dependencies.platform(libs.koin.bom))
            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)
//            implementation(libs.koin.compose.navigation)

            implementation(libs.coil.compose.core)
            implementation(libs.coil.mp)
            implementation(libs.coil.network.ktor)
            implementation(libs.coil.compose)

            implementation(libs.date.picker)

            // file-kit
            implementation(libs.file.kit.core)
            implementation(libs.file.kit.dialogs)
            implementation(libs.file.kit.dialogs.compose)
            implementation(libs.file.kit.coil)

            implementation(libs.androidx.room.runtime)
            implementation(libs.androidx.sqlite.bundled)
            implementation(libs.kmqtt.client)
        }
        commonTest.dependencies { implementation(libs.kotlin.test) }
        iosMain.dependencies {
            implementation(libs.androidx.core.ktx)
            implementation(libs.ktor.client.darwin)
        }
        jvmMain.dependencies {
            //            implementation(libs.androidx.core.ktx)
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)
            implementation(libs.ktor.client.cio)
        }
    }
}

// composeApp/build.gradle.kts
compose.resources {
    packageOfResClass = "arta.composeapp.generated.resources"
    publicResClass = true // Ensures it is visible across your project
    generateResClass = auto
}

room { schemaDirectory("$projectDir/schemas") }

android {
    namespace = "id.nomi.trianapp"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    buildFeatures { androidResources {} }

    defaultConfig {
        applicationId = "id.nomi.trianapp"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = System.getenv("VERSION_CODE")?.toInt() ?: 2
        versionName = System.getenv("VERSION_NAME") ?: "0.0.1"
        multiDexEnabled = true
    }
//
//    val keystorePropertiesFile = rootProject.file("keystore.properties")
//    val keystoreProperties = Properties()
//    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
//
//    signingConfigs {
//        create("release")
//        getByName("release") {
//            storeFile = file("./arta.jks")
//            storePassword = keystoreProperties.getProperty("KEYSTORE_PASSWORD")
//            keyAlias = keystoreProperties.getProperty("KEYSTORE_ALIAS")
//            keyPassword = keystoreProperties.getProperty("KEYSTORE_ALIAS_PASSWORD")
//        }
//    }
    buildTypes {
//        getByName("release") {
//            signingConfig = signingConfigs.getByName("release")
//            isMinifyEnabled = true
//            isDebuggable = false
//            proguardFiles(
//                getDefaultProguardFile("proguard-android-optimize.txt"),
//                "proguard-rules.pro"
//            )
//        }
//        getByName("debug") {
//            signingConfig = signingConfigs.getByName("release")
//            isMinifyEnabled = false
//            isDebuggable = true
//            proguardFiles(
//                getDefaultProguardFile("proguard-android-optimize.txt"),
//                "proguard-rules.pro"
//            )
//        }
    }

    packaging {
        resources { excludes += "/META-INF/{AL2.0,LGPL2.1}" }
        jniLibs {
            keepDebugSymbols +=
                listOf(
                    "**/libandroidx.graphics.path.so",
                    "**/libdatastore_shared_counter.so",
                    "**/libsqliteJni.so"
                )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

dependencies {
    debugImplementation(libs.ui.tooling)
    add("kspAndroid", libs.androidx.room.compiler)
    add("kspIosSimulatorArm64", libs.androidx.room.compiler)
    add("kspIosX64", libs.androidx.room.compiler)
    add("kspIosArm64", libs.androidx.room.compiler)
    add("kspJvm", libs.androidx.room.compiler)
//    // Add any other platform target you use in your project, for example kspDesktop

}

compose.desktop {
    application {
        mainClass = "app.trian.arta.rpc.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "app.trian.arta.rpc"
            packageVersion = "1.0.0"
        }
    }
}
