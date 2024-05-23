{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-23.11";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";

    devshell.url = "github:numtide/devshell";
    devshell.inputs.nixpkgs.follows = "nixpkgs";
    devshell.inputs.flake-utils.follows = "flake-utils";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

    pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks-nix.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit-hooks-nix.inputs.nixpkgs-stable.follows = "nixpkgs-stable";
    pre-commit-hooks-nix.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
    devshell,
    treefmt-nix,
    pre-commit-hooks-nix,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        devshell.flakeModule
        treefmt-nix.flakeModule
        pre-commit-hooks-nix.flakeModule
      ];

      perSystem = {
        config,
        pkgs,
        lib,
        ...
      }: {
        devshells.default = {
          env = [
            {
              name = "LIBCLANG_PATH";
              value = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_latest.libclang.lib];
            }
            {
              name = "LD_LIBRARY_PATH";
              value = pkgs.lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib];
            }
          ];

          packages = [
            pkgs.rustup
            pkgs.nodePackages.pnpm
            pkgs.nodejs
            pkgs.ktlint
            pkgs.pandoc
            pkgs.just
            pkgs.bun
            pkgs.upx
            pkgs.ktfmt
            pkgs.openjdk11
            pkgs.static-web-server
            pkgs.bazel-buildtools
            pkgs.act
            (pkgs.buildFHSEnv {
              name = "ibazel";
              targetPkgs = pkgs: (with pkgs; [zlib.dev libxcrypt bazel_7 bazel-watcher stdenv.cc openjdk11 python3 unzip zip]);
              runScript = "ibazel";
              unsharePid = false;
              unshareUser = false;
              unshareIpc = false;
              unshareNet = false;
              unshareUts = false;
              unshareCgroup = false;
            })
            (pkgs.buildFHSEnv {
              name = "dummy";
              targetPkgs = pkgs: (with pkgs; [zlib.dev libxcrypt bazel_7 stdenv.cc openjdk11 python3 unzip pandoc zip bazelisk libcxxabi.dev glibc]);
              runScript = "bash";
              unsharePid = false;
              unshareUser = false;
              unshareIpc = false;
              unshareNet = false;
              unshareUts = false;
              unshareCgroup = false;
            })
            (pkgs.buildFHSEnv {
              name = "bazel";
              targetPkgs = pkgs: (with pkgs; [zlib.dev libxcrypt bazel_7 stdenv.cc openjdk11 python3 unzip pandoc zip bazelisk libcxxabi.dev]);
              runScript = "/home/flyxi/.cache/bazelisk/downloads/sha256/0a96c4f0a121417e0aad666a3a3a1ef1d72de388463f9f4ddbb496b52e1a0232/bin/bazel";
              multiPkgs = pkgs: (with pkgs; [glibc]);
              unsharePid = false;
              unshareUser = false;
              unshareIpc = false;
              unshareNet = false;
              unshareUts = false;
              unshareCgroup = false;
            })
            (pkgs.buildFHSEnv {
              name = "bazelisk";
              targetPkgs = pkgs: (with pkgs; [zlib.dev libxcrypt bazel_7 stdenv.cc openjdk11 python3 unzip pandoc zip bazelisk android-tools]);
              runScript = "bazelisk";
              unsharePid = false;
              unshareUser = false;
              unshareIpc = false;
              unshareNet = false;
              unshareUts = false;
              unshareCgroup = false;
            })
          ];

          commands = [
            {
              command = "${pkgs.buf}/bin/buf lint";
              name = "buflint";
              help = "lint proto files";
            }
            {
              package = config.treefmt.build.wrapper;
              name = "treefmt";
            }
            {
              command = "${pkgs.rustup}/bin/cargo clippy --all-features";
              name = "clippy";
              help = "lint rust files";
            }
          ];

          devshell.startup.pre-commit.text = config.pre-commit.installationScript;
        };

        treefmt = {
          projectRootFile = ".git/config";
          programs = {
            alejandra.enable = true;
            rustfmt.enable = true;
            prettier = {
              enable = true;
            };
          };

          settings.formatter = {
            buf = {
              command = pkgs.writeShellScriptBin "buf.sh" ''
                for f in $@; do
                  ${pkgs.buf}/bin/buf format --exit-code > /dev/null "$f" || ${pkgs.buf}/bin/buf format -w "$f"
                done
              '';
              includes = ["*.proto"];
            };

            taplo = {
              command = "${pkgs.taplo}/bin/taplo";
              options = [
                "fmt"
                "-o"
                "reorder_keys=true"
                "-o"
                "reorder_arrays=true"
              ];
              includes = ["*.toml"];
            };

            ktfmt = {
              command = "${pkgs.ktfmt}/bin/ktfmt";
              options = [
                "--kotlinlang-style"
              ];
              includes = ["*.kt" "*.kts"];
            };

            buildifier = {
              command = "${pkgs.bazel-buildtools}/bin/buildifier";
              includes = ["BUILD" "WORKSPACE" "MODULE" "*.bzl" "*.bazel" "*.bzlmod"];
            };
          };

          flakeCheck = false;
        };

        pre-commit = {
          settings = {
            hooks.treefmt.enable = true;

            hooks.buf-lint = {
              enable = true;
              name = "Buf Lint";
              entry = "${pkgs.buf}/bin/buf lint";
              types = ["proto"];
              pass_filenames = false;
            };
          };
        };
      };

      systems = [
        "x86_64-linux"
      ];
    };
}
