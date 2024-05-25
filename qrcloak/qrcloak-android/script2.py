versions = dict(
    accompanist = "0.35.0-alpha",
    activity = "1.9.0",
    camera = "1.4.0-alpha05",
    compose_bom = "2024.05.00",
    compose_compiler = "1.5.13",
    concurrent = "1.1.0",
    core = "1.13.1",
    datastore = "1.1.0",
    fragment = "1.7.0",
    lifecycle = "2.7.0",
    mlkit = "17.2.0",
    qrose = "1.0.1",
    voyager = "1.0.0",
)

artifacts = [
    "org.jetbrains.kotlin:kotlin-stdlib:1.9.0",
    "org.jetbrains.kotlin:kotlin-stdlib-jdk8:1.9.0",
    "org.jetbrains.kotlin:kotlin-stdlib-jdk7:1.9.0",
    "org.jetbrains.kotlin:kotlin-reflect:1.9.0",
    "androidx.collection:collection:1.4.0",
    "androidx.collection:collection-jvm:1.4.0",
    "androidx.collection:collection-ktx:1.4.0",
    "net.java.dev.jna:jna:aar:5.14.0",
    "androidx.activity:activity:{}".format(versions["activity"]),
    "androidx.activity:activity-ktx:{}".format(versions["activity"]),
    "androidx.activity:activity-ktx:aar:{}".format(versions["activity"]),
    "androidx.activity:activity-compose:{}".format(versions["activity"]),
    "androidx.compose.compiler:compiler:{}".format(versions["compose_compiler"]),
    "androidx.core:core:{}".format(versions["core"]),
    "androidx.core:core-ktx:{}".format(versions["core"]),
    "androidx.compose.animation:animation",
    "androidx.compose.animation:animation-core",
    "androidx.compose.animation:animation-graphics",
    "androidx.compose.foundation:foundation",
    "androidx.compose.foundation:foundation-layout",
    "androidx.compose.material:material",
    "androidx.compose.material:material-icons-core",
    "androidx.compose.material:material-icons-extended",
    "androidx.compose.material:material-icons-extended-android",
    "androidx.compose.material:material-ripple",
    "androidx.compose.material3:material3",
    "androidx.compose.material3:material3-window-size-class",
    "androidx.compose.runtime:runtime",
    "androidx.compose.runtime:runtime-livedata",
    "androidx.compose.runtime:runtime-rxjava2",
    "androidx.compose.runtime:runtime-rxjava3",
    "androidx.compose.runtime:runtime-saveable",
    "androidx.compose.ui:ui",
    "androidx.compose.ui:ui-geometry",
    "androidx.compose.ui:ui-graphics",
    "androidx.compose.ui:ui-test",
    "androidx.compose.ui:ui-test-junit4",
    "androidx.compose.ui:ui-test-manifest",
    "androidx.compose.ui:ui-text",
    "androidx.compose.ui:ui-text-google-fonts",
    "androidx.compose.ui:ui-tooling",
    "androidx.compose.ui:ui-tooling-data",
    "androidx.compose.ui:ui-tooling-preview",
    "androidx.compose.ui:ui-unit",
    "androidx.compose.ui:ui-util",
    "androidx.compose.ui:ui-viewbinding",
    "androidx.concurrent:concurrent-futures:{}".format(versions["concurrent"]),
    "androidx.concurrent:concurrent-futures-ktx:{}".format(versions["concurrent"]),
    "androidx.fragment:fragment:{}".format(versions["fragment"]),
    "androidx.fragment:fragment-ktx:{}".format(versions["fragment"]),
    "androidx.fragment:fragment-ktx:aar:{}".format(versions["fragment"]),
    "androidx.fragment:fragment-compose:{}".format(versions["fragment"]),
    "androidx.fragment:fragment-testing:{}".format(versions["fragment"]),
    "androidx.fragment:fragment-testing-manifest:{}".format(versions["fragment"]),
    "androidx.lifecycle:lifecycle-common:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-common-java8:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-compiler:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-livedata-ktx:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-livedata-core-ktx:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-process:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-reactivestreams-ktx:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-runtime:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-runtime:aar:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-runtime-ktx:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-runtime-testing:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-service:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-viewmodel-ktx:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-viewmodel-compose:{}".format(versions["lifecycle"]),
    "androidx.lifecycle:lifecycle-viewmodel-savedstate:{}".format(versions["lifecycle"]),
    "io.github.alexzhirkevich:qrose:{}".format(versions["qrose"]),
    "androidx.camera:camera-camera2:{}".format(versions["camera"]),
    "androidx.camera:camera-core:{}".format(versions["camera"]),
    "androidx.camera:camera-extensions:{}".format(versions["camera"]),
    "androidx.camera:camera-lifecycle:{}".format(versions["camera"]),
    "androidx.camera:camera-mlkit-vision:{}".format(versions["camera"]),
    "androidx.camera:camera-video:{}".format(versions["camera"]),
    "androidx.camera:camera-view:{}".format(versions["camera"]),
    "com.google.mlkit:barcode-scanning:{}".format(versions["mlkit"]),
    "cafe.adriel.voyager:voyager-navigator-android:{}".format(versions["voyager"]),
    "cafe.adriel.voyager:voyager-screenmodel-android:{}".format(versions["voyager"]),
    "cafe.adriel.voyager:voyager-tab-navigator-android:{}".format(versions["voyager"]),
    "cafe.adriel.voyager:voyager-transitions:{}".format(versions["voyager"]),
    "androidx.datastore:datastore-preferences-android:{}".format(versions["datastore"]),
    "com.google.accompanist:accompanist-permissions:{}".format(versions["accompanist"]),
]
boms = [
    "androidx.compose:compose-bom:{}".format(versions["compose_bom"]),
]

def get_aar(splits):
    return "aar" if len(splits) > 2 and splits[2] == "aar" else None

def get_key(group, name):
    return f"{group}-{name}".replace(":", "-").replace(".", "-")

def get_version(splits):
    if get_aar(splits):
        return splits[3] if len(splits) > 3 else None
    else:
        return splits[2] if len(splits) > 2 else None

def get_version_ref(version):
    if not version:
        return None

    for key, value in versions.items():
        if value == version:
            return key

    return None

def format_version(version, version_ref):
    if version_ref:
        return f", version.ref = \"{version_ref}\""
    elif version:
        return f", version = \"{version}\""
    else:
        return ""


for artifact in artifacts:
    splits = artifact.split(":")
    group = splits[0]
    name = splits[1]
    
    aar = get_aar(splits)
    key = get_key(group, name)
    version = get_version(splits)
    version_ref = get_version_ref(version)

    name = f"{name}:{aar}" if aar else name
    formatted_version = format_version(version, version_ref)
    value = f"{{ group = \"{group}\", name = \"{name}\"{formatted_version} }}"

    print(f"{key} = {value}")



