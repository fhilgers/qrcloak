import os
import zipfile
import argparse


def unzip_files(jar_files, output_directory):
    if not os.path.exists(output_directory):
        os.makedirs(output_directory)

    for jar_file in jar_files:
        with zipfile.ZipFile(jar_file, "r") as zip_ref:
            zip_ref.extractall(output_directory)


def rchmod(dir):
    os.chmod(dir, 0o755)

    for root, dirs, files in os.walk(dir):
        # set perms on sub-directories
        for momo in dirs:
            os.chmod(os.path.join(root, momo), 0o755)

        # set perms on files
        for momo in files:
            os.chmod(os.path.join(root, momo), 0o644)


def main():
    parser = argparse.ArgumentParser(
        description="Unzip all .jar files in a directory recursively and combine the extracted files into a new .jar file."
    )
    parser.add_argument(
        "inputs", nargs="+", type=str, help="The directory to search for .jar files"
    )
    parser.add_argument(
        "--output_directory",
        required=True,
        type=str,
        help="The directory to extract the contents to",
    )
    parser.add_argument(
        "--exclude",
        action="append",
        default=[],
        help="Glob patterns of files or directories to exclude",
    )

    args = parser.parse_args()

    unzip_files(args.inputs, args.output_directory)
    rchmod(args.output_directory)


if __name__ == "__main__":
    main()
