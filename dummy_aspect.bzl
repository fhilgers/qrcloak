# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

Info = provider(
    fields = {
        "files": "aar and jar files to propagate",
    },
)

def _print_aspect_impl(target, ctx):
    files = []
    if (ctx.rule.kind == "aar_import"):
        files.append(ctx.rule.attr.aar[DefaultInfo].files)
    elif (ctx.rule.kind == "jvm_import"):
        files.append(ctx.rule.attr.jars[0][DefaultInfo].files)

    for dep in getattr(ctx.rule.attr, "deps", []):
        files.append(dep[Info].files)

    for export in getattr(ctx.rule.attr, "exports", []):
        files.append(export[Info].files)

    for runtime_dep in getattr(ctx.rule.attr, "runtime_deps", []):
        files.append(runtime_dep[Info].files)

    for plugin in getattr(ctx.rule.attr, "plugins", []):
        files.append(plugin[Info].files)

    for exported_plugin in getattr(ctx.rule.attr, "exported_compiler_plugins", []):
        print(exported_plugin[Info].files)
        files.append(exported_plugin[Info].files)

    files = depset(transitive = files)
    return [
        Info(files = files),
        OutputGroupInfo(
            files = files,
        ),
    ]

print_aspect = aspect(
    implementation = _print_aspect_impl,
    attr_aspects = ["deps", "plugins", "exported_compiler_plugins", "exports", "runtime_deps"],
)
