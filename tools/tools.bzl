# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

load("@aspect_bazel_lib//lib:copy_to_directory.bzl", "copy_to_directory")

def _gather_jars_impl(ctx):
    srcs = ctx.attr.srcs
    full_compile_jars = []
    for src in srcs:
        java_info = src[JavaInfo]

        #full_compile_jars.append(java_info.compile_jars)

        files = []
        for file in java_info.compile_jars.to_list():
            basename = file.basename
            if basename == "header_kotlin-stdlib-1.9.0.jar":
                pass
            elif basename == "header_kotlinx-coroutines-core-1.7.1.jar":
                pass
            elif basename == "header_collection-1.4.0.jar":
                pass
            elif basename == "androidx_compose_ui_ui_resources.jar" or basename == "androidx_compose_ui_ui_android_resources.jar":
                files.append(file)
            elif basename.endswith("resources.jar") and basename.startswith("androidx_compose_"):
                pass
            elif basename == "android-ijar.jar":
                pass
            else:
                files.append(file)

        full_compile_jars.append(depset(files))

    full_compile_jars = depset(transitive = full_compile_jars)

    return [
        DefaultInfo(files = depset(transitive = [full_compile_jars])),
    ]

def _impl(settings, attr):
    _ignore = (settings, attr)
    return {
        "x86": {
            "//command_line_option:platforms": "//:x86",
        },
        "x86_64": {
            "//command_line_option:platforms": "//:x86_64",
        },
        "arm64-v8a": {
            "//command_line_option:platforms": "//:arm64-v8a",
        },
        "armeabi-v7a": {
            "//command_line_option:platforms": "//:armeabi-v7a",
        },
        "linux-x86-64": {
            "//command_line_option:platforms": "//:linux-x86-64",
        },
    }

multi_arch_transition = transition(
    implementation = _impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def get_libs_for_arch(ctx, arch):
    native_libs = []
    for src in ctx.split_attr.srcs[arch]:
        java_info = src[JavaInfo]
        cc_info = java_info.cc_link_params_info

        for linker_input in cc_info.linking_context.linker_inputs.to_list():
            for library in linker_input.libraries:
                native_libs.append(depset([library.resolved_symlink_dynamic_library]))

    native_libs = depset(transitive = native_libs)

    return native_libs

def _gather_cc_impl(ctx):
    libs = ctx.actions.declare_directory(ctx.attr.name + "-android")

    x86 = get_libs_for_arch(ctx, "x86")
    x86_64 = get_libs_for_arch(ctx, "x86_64")
    arm64_v8a = get_libs_for_arch(ctx, "arm64-v8a")
    armeabi_v7a = get_libs_for_arch(ctx, "armeabi-v7a")

    args = ctx.actions.args()
    args.add(libs.path)
    args.add_joined(x86, join_with = " ")
    args.add_joined(x86_64, join_with = " ")
    args.add_joined(arm64_v8a, join_with = " ")
    args.add_joined(armeabi_v7a, join_with = " ")

    ctx.actions.run_shell(
        outputs = [libs],
        arguments = [args],
        inputs = depset(transitive = [x86, x86_64, arm64_v8a, armeabi_v7a]),
        command = """
LIBS_PATH=$1

mkdir -p $LIBS_PATH/lib/x86
mkdir -p $LIBS_PATH/lib/x86_64
mkdir -p $LIBS_PATH/lib/arm64-v8a
mkdir -p $LIBS_PATH/lib/armeabi-v7a

for file in $2; do
  cp -L $file $LIBS_PATH/lib/x86
done
for file in $3; do
  cp -L $file $LIBS_PATH/lib/x86_64
done

for file in $4; do
  cp -L $file $LIBS_PATH/lib/arm64-v8a
done

for file in $5; do
  cp -L $file $LIBS_PATH/lib/armeabi-v7a
done
""",
    )

    desktop = ctx.actions.declare_file(ctx.attr.name + "-linux-x86-64.jar")

    linux_x86_64 = get_libs_for_arch(ctx, "linux-x86-64")

    args = ctx.actions.args()
    args.add(desktop.path)
    args.add_joined(linux_x86_64, join_with = " ")

    ctx.actions.run_shell(
        outputs = [desktop],
        arguments = [args],
        inputs = linux_x86_64,
        command = """
mkdir -p libs

for file in $2; do
  cp -L $file libs/
done

(cd libs && zip -r ../$1 .)
    """,
    )

    libs_zip = ctx.actions.declare_file(ctx.label.name + ".jar")

    args = ctx.actions.args()
    args.add(libs.path)
    args.add(libs_zip)

    ctx.actions.run_shell(
        arguments = [args],
        outputs = [libs_zip],
        inputs = [libs],
        command = "DIR=$PWD && cd $1 && zip -r $DIR/$2 ./*",
    )

    return [
        DefaultInfo(files = depset([libs_zip, desktop])),
    ]

def _gather_natives_impl(ctx):
    srcs = ctx.attr.srcs

    native_libs = []
    for src in srcs:
        if AndroidNativeLibsInfo in src:
            native_libs.append(src[AndroidNativeLibsInfo].native_libs)

    if len(native_libs) == 0:
        return []

    libs = ctx.actions.declare_directory(ctx.label.name)
    libs_zip = ctx.actions.declare_file(ctx.label.name + ".jar")

    native_libs = depset(transitive = native_libs)

    args = ctx.actions.args()
    args.add(libs.path)
    args.add_all(native_libs)

    ctx.actions.run_shell(
        arguments = [args],
        outputs = [libs],
        inputs = native_libs,
        command = "OUT=$1 && shift && for file in $@; do unzip  $file -d $OUT || echo empty; done",
    )

    args = ctx.actions.args()
    args.add(libs.path)
    args.add(libs_zip)

    ctx.actions.run_shell(
        arguments = [args],
        outputs = [libs_zip],
        inputs = [libs],
        command = "DIR=$PWD && cd $1 && zip -r $DIR/$2 ./*",
    )

    return [
        DefaultInfo(files = depset(direct = [libs_zip])),
    ]

gather_natives = rule(
    implementation = _gather_natives_impl,
    attrs = dict(
        srcs = attr.label_list(cfg = multi_arch_transition),
    ),
)

gather_cc = rule(
    implementation = _gather_cc_impl,
    attrs = dict(
        srcs = attr.label_list(cfg = multi_arch_transition),
    ),
)

gather_jars = rule(
    implementation = _gather_jars_impl,
    attrs = dict(
        srcs = attr.label_list(),
    ),
)

def _fat_jar_impl(ctx):
    files = []
    for src in ctx.attr.srcs:
        files.append(src[DefaultInfo].files)

    files = depset(transitive = files)

    outs = []
    for file in files.to_list():
        if file.basename == "classes_and_libs_merged-stamped.jar":
            out_name = file.owner.name + ".jar"
        elif file.basename.startswith("header_"):
            out_name = file.owner.name + ".jar"
        else:
            out_name = file.basename

        out_name = out_name.replace("_", "-")

        out = ctx.actions.declare_file(out_name)
        ctx.actions.symlink(output = out, target_file = file)
        outs.append(out)

    outs = depset(outs)

    jars = ctx.actions.declare_directory(ctx.label.name)

    args = ctx.actions.args()
    args.add(jars.path)
    args.add_all(outs)

    ctx.actions.run_shell(
        arguments = [args],
        inputs = outs,
        outputs = [jars],
        command = """
OUT=$1
shift
for file in $@; do
  cp $file $OUT/
done

chmod -R a=r,u+w,a+X $OUT
""",
    )

    return [
        DefaultInfo(files = depset([jars])),
    ]

fat_jar = rule(
    implementation = _fat_jar_impl,
    attrs = dict(
        srcs = attr.label_list(),
        _unzip_jars = attr.label(
            default = Label("//tools:unzip_jars"),
            cfg = "exec",
            executable = True,
        ),
        _zip = attr.label(
            default = Label("//tools:zip"),
            cfg = "exec",
            executable = True,
        ),
    ),
)

def provide_libs(name, srcs):
    native_libs_name = "{}_native_libs".format(name)
    jars_name = "{}_jars".format(name)
    cc_name = "{}_cc".format(name)

    gather_natives(
        name = native_libs_name,
        srcs = srcs,
    )

    gather_jars(
        name = jars_name,
        srcs = srcs,
    )

    gather_cc(
        name = cc_name,
        srcs = srcs,
    )

    fat_jar(
        name = name,
        srcs = [
            ":{}".format(native_libs_name),
            ":{}".format(jars_name),
            ":{}".format(cc_name),
        ],
    )
