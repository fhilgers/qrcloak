# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _uniffi_bindgen_impl(ctx):
    toolchain = ctx.toolchains["//toolchains/uniffi:toolchain_type"]

    cli = toolchain.uniffi_cli.files_to_run
    cargo = toolchain.cargo.files_to_run

    cargo_sh = ctx.actions.declare_file("cargo")

    ctx.actions.write(cargo_sh, """#!/bin/sh
CARGO=$PWD/{cargo}
cd {basedir} && $CARGO $@
""".format(cargo = cargo.executable.path, basedir = ctx.attr.basedir), is_executable = True)

    output_dir = ctx.actions.declare_directory(ctx.label.name)

    inputs = []
    for src in ctx.attr.srcs:
        inputs.append(src[DefaultInfo].files)

    inputs = depset(transitive = inputs)

    library_files = ctx.attr.library[DefaultInfo].files.to_list()
    if len(library_files) != 1:
        fail("want exactly one file for the library")
    library = library_files[0]

    inputs = depset(direct = [library], transitive = [inputs])

    args = ctx.actions.args()
    args.add("generate")
    args.add("--library", library.path)
    args.add("--out-dir", output_dir.path)
    args.add("--language", ctx.attr.language)

    ctx.actions.run(
        executable = cli,
        inputs = inputs,
        outputs = [output_dir],
        arguments = [args],
        tools = [cargo, cargo_sh],
        env = dict(
            PATH = "/bin:/usr/bin:{}".format(cargo_sh.dirname),
        ),
    )

    return [
        DefaultInfo(
            files = depset(direct = [output_dir]),
        ),
    ]

uniffi_bindgen = rule(
    implementation = _uniffi_bindgen_impl,
    attrs = dict(
        library = attr.label(mandatory = True),
        srcs = attr.label_list(mandatory = True, allow_files = True),
        basedir = attr.string(mandatory = True),
        language = attr.string(mandatory = True),
    ),
    toolchains = ["//toolchains/uniffi:toolchain_type"],
)

def _uniffi_toolchain_impl(ctx):
    toolchain_info = platform_common.ToolchainInfo(
        uniffi_cli = ctx.attr.uniffi_cli[DefaultInfo],
        cargo = ctx.attr.cargo,
    )
    return [toolchain_info]

uniffi_toolchain = rule(
    implementation = _uniffi_toolchain_impl,
    attrs = dict(
        uniffi_cli = attr.label(
            mandatory = True,
            executable = True,
            cfg = "exec",
        ),
        cargo = attr.label(
            mandatory = True,
            executable = True,
            cfg = "exec",
        ),
    ),
)
