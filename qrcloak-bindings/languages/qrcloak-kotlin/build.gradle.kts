import com.android.build.gradle.internal.tasks.factory.dependsOn
import org.gradle.configurationcache.extensions.capitalized

plugins {
    kotlin("multiplatform") version "1.9.23"
    id("maven-publish")
    id("com.android.library") version "8.2.0"
}

group = "com.github.fhilgers"

version = "1.0-SNAPSHOT"

publishing { repositories { mavenLocal() } }

kotlin {
    jvm {
        jvmToolchain(11)
        testRuns["test"].executionTask.configure { useJUnitPlatform() }
    }
    androidTarget { publishLibraryVariants("release", "debug") }

    sourceSets {
        val commonMain by getting {}
        val commonTest by getting { dependencies { implementation(kotlin("test")) } }
        val jvmMain by getting { dependencies { implementation("net.java.dev.jna:jna:5.14.0") } }

        val jvmTest by getting
        val androidMain by getting {
            dependencies { implementation("net.java.dev.jna:jna:5.14.0@aar") }
        }
        val androidUnitTest by getting { dependencies { implementation("junit:junit:4.13.2") } }
    }
}

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
val srcDirs = listOf(File("../../src"), File("../../../src"))
val libName = "libqrcloak_bindings.so"

fun buildRelease(target: String?): TaskProvider<Exec> =
    tasks.register<Exec>("cargoBuild${target?:"Current"}Lib") {
        group = "cargo"

        inputs.dir("../../src")

        val args =
            listOf(
                "cargo",
                "build",
                "--release",
            )

        val outputLib = target?.let { "${it}/release/${libName}" } ?: "release/${libName}"

        outputs.file(File(targetDir, outputLib))

        commandLine(target?.let { args + listOf("--target", it) } ?: args)
    }

val cargoBuildCurrentTarget = buildRelease(null)
val cargoAndroidTargets = androidTargets.map(::buildRelease).toList()
val cargoDesktopTargets = desktopTargets.map(::buildRelease).toList()

val cargoBuildAndroidLibs =
    tasks.register("cargoBuildAndroidLibs") {
        group = "cargo"

        dependsOn(cargoAndroidTargets)

        outputs.files(cargoAndroidTargets.flatMap { it.get().outputs.files })
    }

val cargoBuildDesktopLibs =
    tasks.register("cargoBuildDesktopLibs") {
        group = "cargo"

        dependsOn(cargoDesktopTargets)

        outputs.files(cargoDesktopTargets.flatMap { it.get().outputs.files })
    }

val cargoBuildAllLibs =
    tasks.register("cargoBuildAllLibs") {
        group = "cargo"

        dependsOn(cargoBuildAndroidLibs, cargoBuildDesktopLibs, cargoBuildCurrentTarget)
    }

val uniffiBindgen =
    tasks.register<Exec>("uniffiBindgen") {
        group = "cargo"

        dependsOn(cargoBuildCurrentTarget)

        val libFile = cargoBuildCurrentTarget.get().outputs.files.singleFile

        val outDir =
            File(layout.buildDirectory.asFile.get(), "generated/source/uniffi/commonMain/kotlin")
        outputs.dir(outDir)

        commandLine(
            "cargo",
            "run",
            "--features=uniffi/cli",
            "--bin",
            "uniffi-bindgen",
            "generate",
            "--library",
            "$libFile",
            "--language",
            "kotlin",
            "--out-dir",
            outDir.path
        )
    }

val bundleAndroidLibs =
    tasks.register("bundleAndroidLibs") {
        group = "cargo"

        dependsOn(cargoBuildAndroidLibs)

        val libFiles = cargoBuildAndroidLibs.get().outputs.files

        val jniLibs = File(layout.buildDirectory.asFile.get(), "generated/jniLibs")
        outputs.dir(jniLibs)

        doLast {
            libFiles.zip(jniLibMappings).forEach { (libFile, dirPrefix) ->
                copy {
                    from(libFile)
                    into(File(jniLibs, dirPrefix))
                }
            }
        }
    }

val bundleJvmLibs =
    tasks.register("bundleJvmLibs") {
        group = "cargo"

        dependsOn(cargoBuildDesktopLibs)

        val libFiles = cargoBuildDesktopLibs.get().outputs.files

        val libs = File(layout.buildDirectory.asFile.get(), "generated/jvmLibs")
        outputs.dir(libs)

        doLast {
            libFiles.zip(desktopLibMappings).forEach { (libFile, dirPrefix) ->
                copy {
                    from(libFile)
                    into(File(libs, dirPrefix))
                }
            }
        }
    }

val bundleJvmLibsJar =
    tasks.register<Jar>("bundleJvmLibsJar") {
        group = "cargo"

        archiveBaseName = "qrcloak-core-jvm-libs"

        from(bundleJvmLibs)
    }

publishing {
    publications {
        create<MavenPublication>("jvmLibs") {
            artifactId = "qrcloak-jvm-libs"

            artifact(bundleJvmLibsJar)
        }
    }
}

afterEvaluate {
    kotlin.sourceSets.getByName("commonMain").kotlin.srcDir(uniffiBindgen)
    kotlin.sourceSets.getByName("jvmMain").resources.srcDir(bundleJvmLibs)

    android.libraryVariants.forEach { variant ->
        val t =
            tasks.register("bundleAndroid${variant.name.capitalized()}Libs") {
                dependsOn(bundleAndroidLibs)
                android.sourceSets
                    .getByName(variant.name)
                    .jniLibs
                    .srcDir(bundleAndroidLibs.get().outputs.files)
            }
        tasks.named("merge${variant.name.capitalized()}JniLibFolders").dependsOn(t)
    }
}
