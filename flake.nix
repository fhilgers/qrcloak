# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT
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
              targetPkgs = pkgs: (with pkgs; [zlib.dev bazel-watcher libxcrypt bazel_7 stdenv.cc openjdk11 python3 unzip pandoc zip bazelisk android-tools]);
              runScript = "ibazel";
              unsharePid = false;
              unshareUser = false;
              unshareIpc = false;
              unshareNet = false;
              unshareUts = false;
              unshareCgroup = false;
            })
            (pkgs.buildFHSEnv {
              name = "bazel";
              targetPkgs = pkgs: (with pkgs; [zlib.dev clang libxcrypt bazel_7 stdenv.cc openjdk11 python3 unzip pandoc zip bazelisk android-tools]);
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
            biome.enable = true;
            ruff = {
              check = true;
              format = true;
            };
            shfmt.enable = true;
            buildifier = {
              enable = true;
              includes = ["BUILD" "WORKSPACE" "MODULE" "*.bzl" "*.bazel" "*.bzlmod"];
            };
          };

          settings.formatter = {
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
          };

          flakeCheck = false;
        };

        pre-commit = {
          settings = {
            hooks.treefmt.enable = true;
          };
        };
      };

      systems = [
        "x86_64-linux"
      ];
    };
}
