import pathlib
import sys

if len(sys.argv) != 3:
    print("Usage: python gen_khronos_docs.py <path_to_khronos_repo> <path_to_output_file>")
    sys.exit(1) 

# Delete the output file if it exists
if pathlib.Path(sys.argv[2]).exists():
    pathlib.Path(sys.argv[2]).unlink()

for path in sorted(pathlib.Path(sys.argv[1]).rglob("README.md")):
    if path.parent == pathlib.Path(sys.argv[1]):
        continue
    print(path)
    with open(path, "r") as f:
        with open(sys.argv[2], "a") as output:
            output.write(f.read())
            output.write("\n---\n")
