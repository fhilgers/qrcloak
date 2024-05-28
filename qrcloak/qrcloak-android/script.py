import tomllib

TEMPLATE = """
maven_versions = {versions}
maven_artifacts = {artifacts}
maven_boms = {boms}


maven = use_extension("@rules_jvm_external//:extensions.bzl", "maven")
maven.install(
    name = "maven_deps",
    artifacts = maven_artifacts,
    boms = maven_boms,
    fail_if_repin_required = True,
    lock_file = "//:manifest_install.json",
    repositories = [
        "https://maven.google.com",
        "https://repo1.maven.org/maven2",
    ],
    resolver = "maven",
    use_starlark_android_rules = True,
    aar_import_bzl_label = "@rules_android//android:rules.bzl",
)
use_repo(maven, "maven_deps")
""".strip()


def quote(s):
    return f'"{s}"'


def ident(s, by):
    return f"{by}{s}"


def format(ss):
    joined = ",\n".join(ss)
    return f"[\n{joined}\n]"


def format_versions(mv):
    lines = []
    for key, val in mv.items():
        lines.append(f"    {key} = {quote(val)}")

    joined = ",\n".join(lines)
    return f"dict(\n{joined}\n)"


boms = []
artifacts = []
versions = []

with open("./gradle/libs.versions.toml", "rb") as f:
    data = tomllib.load(f)
    versions = data["versions"]
    libraries = data["libraries"]
    for library in libraries.values():
        if "module" in library:
            module = library["module"]
        else:
            group = library["group"]
            name = library["name"]
            module = f"{group}:{name}"

        version_ref = library.get("version", {}).get("ref")
        version = versions.get(version_ref) if version_ref else None

        full = (
            f'    "{module}:{{}}".format(maven_versions["{version_ref}"])'
            if version_ref
            else f'    "{module}"'
        )

        if module == "androidx.compose:compose-bom":
            boms.append(full)
        else:
            artifacts.append(full)

print(
    TEMPLATE.format(
        artifacts=format(artifacts),
        boms=format(boms),
        versions=format_versions(versions),
    )
)
