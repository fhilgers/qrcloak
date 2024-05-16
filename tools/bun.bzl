load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _bun_impl(ctx):
    ctx.download_and_extract(
        "https://github.com/oven-sh/bun/releases/download/bun-v1.1.8/bun-linux-x64-baseline.zip",
    )

    ctx.download_and_extract(
        "https://github.com/oven-sh/bun/releases/download/bun-v1.1.8/bun-windows-x64-baseline.zip",
    )

    ctx.file("BUILD.bazel", content = """

config_setting(
    name = "x86_64-linux",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
)

config_setting(
    name = "x86_64-windows",
    constraint_values = [
        "@platforms//os:windows",
        "@platforms//cpu:x86_64",
    ],
)

alias(
  name = "bin",
  actual = select({
    ":x86_64-linux": "bun-linux-x64-baseline/bun",
    ":x86_64-windows": "bun-windows-x64-baseline/bun.exe",
  }),
  visibility = ["//visibility:public"]
)
    """)

bun_repo = repository_rule(
    implementation = _bun_impl,
    attrs = {
    },
)

def _my_extension_impl(ctx):
    bun_repo(name = "bun")

bun = module_extension(implementation = _my_extension_impl)

def _bun_test_impl(ctx):
    pass

bun_test = rule(
    implementation = _bun_test_impl,
    attrs = dict(
        _bun = attr.label(
            default = "@bun//:bin",
            executable = True,
            cfg = "exec",
        ),
    ),
    test = True,
)
