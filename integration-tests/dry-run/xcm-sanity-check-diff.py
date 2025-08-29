import re
import sys
from pathlib import Path

def normalize_line(line: str) -> str:
    """Remove leading checkmark/cross and whitespace for sorting."""
    return re.sub(r'^[✅❌\s]+', '', line)

def normalize_file(path):
    """Reads file, sorts lines, and returns sorted list of lines."""
    with open(path) as f:
        lines = [line.strip() for line in f if line.strip()]
    return sorted(lines, key=normalize_line)

def extract_numbers(line):
    """Extracts all integers/floats from a line."""
    return [float(x) if "." in x else int(x) for x in re.findall(r"\d+\.\d+|\d+", line)]

def compare_files(file1, file2):
    lines1 = normalize_file(file1)
    lines2 = normalize_file(file2)

    # align by sorted order
    max_len = max(len(lines1), len(lines2))
    for i in range(max_len):
        l1 = lines1[i] if i < len(lines1) else ""
        l2 = lines2[i] if i < len(lines2) else ""

        #if l1 == l2:
        #    continue  # identical line, skip

        nums1 = extract_numbers(l1)
        nums2 = extract_numbers(l2)

        print(f"\nLine {i+1}:")
        print(f"  {file1}: {l1}")
        print(f"  {file2}: {l2}")

        if nums1 and nums2 and len(nums1) == len(nums2):
            diffs = [b - a for a, b in zip(nums1, nums2)]
            print(f"  Differences: {diffs}")
        else:
            print("  Non-matching or missing numbers.")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} file1 file2")
        sys.exit(1)
    compare_files(sys.argv[1], sys.argv[2])
