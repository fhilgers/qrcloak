plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.jetbrains.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "com.github.fhilgers.qrcloak"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.github.fhilgers.qrcloak"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables { useSupportLibrary = true }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
    buildFeatures { compose = true }
    composeOptions { kotlinCompilerExtensionVersion = "1.5.13" }
    packaging { resources { excludes += "/META-INF/{AL2.0,LGPL2.1}" } }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui.ui)
    implementation(libs.androidx.compose.ui.ui.graphics)
    implementation(libs.androidx.compose.ui.ui.tooling.preview)
    implementation(libs.androidx.compose.material3.material3)
    implementation(libs.androidx.compose.material.material.icons.extended)
    implementation(libs.androidx.compose.animation.animation)
    implementation(libs.androidx.compose.foundation.foundation)
    implementation(libs.androidx.core.core.ktx)
    implementation(libs.androidx.activity.activity.compose)

    implementation(libs.cafe.adriel.voyager.voyager.navigator.android)
    implementation(libs.cafe.adriel.voyager.voyager.screenmodel.android)
    implementation(libs.cafe.adriel.voyager.voyager.tab.navigator.android)
    implementation(libs.cafe.adriel.voyager.voyager.transitions)

    implementation(libs.com.google.accompanist.accompanist.permissions)
    implementation(libs.com.google.mlkit.barcode.scanning)

    implementation(libs.androidx.camera.camera.mlkit.vision)
    implementation(libs.androidx.camera.camera.core)
    implementation(libs.androidx.camera.camera.extensions)
    implementation(libs.androidx.camera.camera.lifecycle)
    implementation(libs.androidx.camera.camera.video)
    implementation(libs.androidx.camera.camera.view)
    implementation(libs.androidx.camera.camera.camera2)

    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.org.roboelectric.roboelectric)
    androidTestImplementation(libs.androidx.compose.ui.ui.test.junit4)
    androidTestImplementation(libs.androidx.compose.ui.ui.test.manifest)
}
