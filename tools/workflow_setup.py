# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

import os
import subprocess
import urllib.request
import hashlib
import zipfile
from tqdm import tqdm
from textwrap import dedent
from typing import TYPE_CHECKING, NoReturn

custom_bar_format = "[{bar:39}] {percentage:3.0f}% {desc}"


def install_jre():
    e = { 
        #"DEBIAN_FRONTEND": "noninteractive",
    }
    os.symlink("/usr/share/zoneinfo/Europe/Berlin", "/etc/localtime")
    with open("/etc/timezone", "w") as tz:
        tz.write("Europe/Berlin")

    subprocess.run(
        ["sudo", "apt-get", "update"], 
        check=True,
        env=e
    )
    subprocess.run(
        ["sudo", "apt-get", "install", "--yes", "openjdk-17-jre"],
        check=True,
        env=e
    )


def check_sha256(filename: str, expected_sha256: str):
    sha256_hash = hashlib.sha256()
    with open(filename, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest() == expected_sha256


class DownloadProgressBar(tqdm[NoReturn] if TYPE_CHECKING else tqdm):
    def update_to(self, b: int, bsize: int, tsize: int):
        self.total = tsize

        previous = self.n
        current = min(b * bsize, tsize)

        self.update(current - previous)


def download_with_progress(url: str, filename: str):
    with DownloadProgressBar(
        desc=filename, bar_format=custom_bar_format, ascii=" ="
    ) as t:
        urllib.request.urlretrieve(url, filename, reporthook=t.update_to)
        t.close()


def unzip_with_progress(zip_file: str, extract_to: str):
    with zipfile.ZipFile(zip_file, "r") as zip_ref:
        total_files = len(zip_ref.infolist())
        with tqdm(
            total=total_files,
            unit="file",
            desc="Unzipping",
            bar_format=custom_bar_format,
            ascii=" =",
        ) as t:
            for file in zip_ref.infolist():
                zip_ref.extract(file, extract_to)
                file_path = os.path.join(extract_to, file.filename)
                # Restore file permissions
                perm = file.external_attr >> 16
                os.chmod(file_path, perm)
                t.update(1)


def main():
    install_jre()

    VERSION_LONG = "10406996"
    FILENAME = f"commandlinetools-linux-{VERSION_LONG}_latest.zip"
    SHA256 = "8919e8752979db73d8321e9babe2caedcc393750817c1a5f56c128ec442fb540"
    ANDROID_SDK_ROOT = os.path.expanduser("~/.android/sdk")

    if not os.path.isdir(ANDROID_SDK_ROOT):
        url = f"https://dl.google.com/android/repository/{FILENAME}"
        download_with_progress(url, FILENAME)

        if not check_sha256(FILENAME, SHA256):
            raise ValueError("SHA256 checksum does not match")

        os.makedirs(f"{ANDROID_SDK_ROOT}/cmdline-tools/", exist_ok=True)

        unzip_with_progress(FILENAME, f"{ANDROID_SDK_ROOT}/tmp")

        os.rename(
            f"{ANDROID_SDK_ROOT}/tmp/cmdline-tools",
            f"{ANDROID_SDK_ROOT}/cmdline-tools/latest",
        )

    subprocess.run(
        [f"{ANDROID_SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager", "--licenses"],
        input=b"y\n" * 100,
        capture_output=True,
        check=True,
    )
    subprocess.run(
        [
            f"{ANDROID_SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager",
            "ndk;26.3.11579264",
            "build-tools;34.0.0",
            "platforms;android-34",
        ],
        check=True,
    )

    ANDROID_HOME = ANDROID_SDK_ROOT
    ANDROID_NDK_HOME = f"{ANDROID_HOME}/ndk/26.3.11579264"

    ci_bazelrc_content = dedent(
        f"""\
    common --repo_env=ANDROID_HOME={ANDROID_HOME}
    common --repo_env=ANDROID_NDK_HOME={ANDROID_NDK_HOME}
    """
    )

    BUILD_WORKSPACE_DIRECTORY = os.getenv("BUILD_WORKSPACE_DIRECTORY", ".")
    with open(os.path.join(BUILD_WORKSPACE_DIRECTORY, "ci.bazelrc"), "w") as f:
        f.write(ci_bazelrc_content)


if __name__ == "__main__":
    main()
