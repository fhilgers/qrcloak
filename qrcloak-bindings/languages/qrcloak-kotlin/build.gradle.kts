import com.ncorti.ktfmt.gradle.tasks.KtfmtFormatTask

plugins {
    kotlin("multiplatform") version "1.8.21"
    id("maven-publish")
    id("com.android.library") version "8.3.2"

    id("com.ncorti.ktfmt.gradle") version "0.18.0"

    idea
}

group = "org.example"

version = "1.0-SNAPSHOT-0"

repositories {
    google()
    mavenCentral()
}

publishing { repositories { mavenLocal() } }

kotlin {
    jvm {
        jvmToolchain(11)
        testRuns["test"].executionTask.configure { useJUnitPlatform() }
    }
    android { publishLibraryVariants("release", "debug") }

    sourceSets {
        val commonMain by getting { dependencies {} }
        val commonTest by getting { dependencies { implementation(kotlin("test")) } }
        val jvmMain by getting { dependencies { implementation("net.java.dev.jna:jna:5.14.0") } }

        val jvmTest by getting
        val androidMain by getting {
            dependencies { implementation("net.java.dev.jna:jna:5.14.0@aar") }
        }
        val androidUnitTest by getting { dependencies { implementation("junit:junit:4.13.2") } }

    }
}

tasks.register<KtfmtFormatTask>("ktfmt") {
    source = project.fileTree(rootDir)
    include("**/*.kt")
    include("**/*.kts")
}

ktfmt { kotlinLangStyle() }

android {
    namespace = "org.example.library"
    compileSdk = 34
    defaultConfig { minSdk = 24 }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

val desktopTargets = listOf("x86_64-unknown-linux-gnu")
val desktopLibMappings = listOf("linux-x86-64")

val androidTargets =
    listOf(
        "x86_64-linux-android",
        "i686-linux-android",
        "armv7-linux-androideabi",
        "aarch64-linux-android"
    )
val jniLibMappings = listOf("x86_64", "x86", "armeabi-v7a", "arm64-v8a")

val rootDir = File("../../../")
val targetDir = File(rootDir.path, "target")

val build =
    tasks.create("cargo-build") {
        doLast {
            (desktopTargets + androidTargets).forEach { target ->
                exec { commandLine("cargo", "build", "--release", "--target", target) }
            }
        }
    }

tasks.create("cargo-deploy") {
    dependsOn(build)

    val jniLibs = File(layout.buildDirectory.asFile.get(), "generated/jniLibs")
    val desktopLibs = File(layout.buildDirectory.asFile.get(), "generated/desktopLibs")

    android.sourceSets.getByName("main").jniLibs.srcDir(jniLibs)
    kotlin.sourceSets.getByName("jvmMain").resources.srcDir(desktopLibs)

    doLast {
        androidTargets.zip(jniLibMappings).forEach { (target, dir) ->
            sync {
                from(File(targetDir.path, "${target}/release"))
                include("*.so")
                into(File(jniLibs, dir))
            }
        }

        desktopTargets.zip(desktopLibMappings).forEach { (target, dir) ->
            sync {
                from(File(targetDir.path, "${target}/release"))
                include("*.so")
                into(File(desktopLibs, dir))
            }
        }
    }
}

tasks.create("uniffi", Exec::class.java) {
    dependsOn(build)

    val outDir =
        File(layout.buildDirectory.asFile.get(), "generated/source/uniffi/commonMain/kotlin")
    val sourceSet = kotlin.sourceSets.getByName("commonMain")
    sourceSet.kotlin.srcDir(outDir)

    idea.module.generatedSourceDirs.add(outDir)

    commandLine(
        "cargo",
        "run",
        "--features=uniffi/cli",
        "--bin",
        "uniffi-bindgen",
        "generate",
        "--library",
        "../../../target/release/libqrcloak_bindings.so",
        "--language",
        "kotlin",
        "--out-dir",
        outDir.path
    )
}

tasks.named("preBuild") {
    dependsOn("uniffi")
    dependsOn("cargo-deploy")
}
