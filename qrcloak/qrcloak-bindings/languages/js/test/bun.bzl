load("@aspect_rules_js//npm:providers.bzl", "NpmPackageInfo")
load("@aspect_bazel_lib//lib:copy_file.bzl", "COPY_FILE_TOOLCHAINS", "copy_file_action")

def _bun_test_impl(ctx):
    runfiles = ctx.runfiles(files = ctx.files.data + ctx.files._runfiles)

    outs = []
    for data in ctx.attr.data:
        if NpmPackageInfo in data:
            info = data[NpmPackageInfo]
            package = info.package
            directory = info.directory
            out = ctx.actions.declare_directory(ctx.label.name + "/node_modules/" + package)
            ctx.actions.symlink(output = out, target_file = directory)
            outs.append(out)

    entry = ctx.actions.declare_file(ctx.label.name + "/" + ctx.attr.entry_point[DefaultInfo].files.to_list()[0].basename)
    copy_file_action(ctx, ctx.attr.entry_point[DefaultInfo].files.to_list()[0], entry)

    bun = ctx.attr._bun[DefaultInfo].files_to_run.executable
    outs.append(entry)
    outs.append(bun)

    node_modules = ctx.runfiles(files = outs)

    runfiles = runfiles.merge(node_modules)

    entry_path = ctx.workspace_name + "/" + entry.short_path
    bun_path = ctx.workspace_name + "/" + bun.short_path

    out = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(
        output = out,
        content =
            """\
#!/usr/bin/env bash
# --- begin runfiles.bash initialization v3 ---
# Copy-pasted from the Bazel Bash runfiles library v3.
set -uo pipefail; set +e; f=bazel_tools/tools/bash/runfiles/runfiles.bash
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null || \
  source "$0.runfiles/$f" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null || \
  { echo>&2 "ERROR: cannot find $f"; exit 1; }; f=; set -e
# --- end runfiles.bash initialization v3 ---

DIR="$(dirname $(rlocation "%s"))"
BUN="$(rlocation %s)"

cd $DIR && $BUN test
""" % (entry_path, bun_path),
        is_executable = True,
    )

    return [
        DefaultInfo(executable = out, runfiles = runfiles),
    ]

bun_test = rule(
    implementation = _bun_test_impl,
    attrs = {
        "entry_point": attr.label(mandatory = True, allow_files = [".ts", ".js"]),
        "data": attr.label_list(providers = [NpmPackageInfo]),
        "_bun": attr.label(default = "@bun//:bin", allow_files = True),
        "_runfiles": attr.label(default = "@bazel_tools//tools/bash/runfiles"),
    },
    test = True,
    toolchains = COPY_FILE_TOOLCHAINS,
)
