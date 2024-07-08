# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

import os
import re
from typing import List
import zipfile
import argparse


def should_exclude(file_path: str, excludes: List[str], base_directory: str):
    relative_path = os.path.relpath(file_path, base_directory)
    for pattern in excludes:
        if re.match(pattern, relative_path):
            return True
    return False


def zip_directory(directory: str, zip_filename: str, excludes: List[str]):
    with zipfile.ZipFile(zip_filename, "w", zipfile.ZIP_DEFLATED) as jar_file:
        for root, _, files in os.walk(directory):
            for file in files:
                file_path = os.path.join(root, file)
                arcname = os.path.relpath(file_path, start=directory)
                if not should_exclude(file_path, excludes, directory):
                    jar_file.write(file_path, arcname)


def main():
    parser = argparse.ArgumentParser(
        description="Unzip all .jar files in a directory recursively and combine the extracted files into a new .jar file."
    )
    parser.add_argument(
        "input_directory", type=str, help="The directory to search for .jar files"
    )
    parser.add_argument("output_file", type=str, help="The path to the final .jar file")
    parser.add_argument(
        "--exclude",
        action="append",
        default=[],
        help="Glob patterns of files or directories to exclude",
    )

    args = parser.parse_args()

    zip_directory(args.input_directory, args.output_file, args.exclude)


if __name__ == "__main__":
    main()
