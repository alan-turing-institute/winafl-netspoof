import sys
import os


def read_and_print_files(directory_path):
    if not os.path.isdir(directory_path):
        print(f"Error: '{directory_path}' is not a valid directory.")
        return

    for filename in os.listdir(directory_path):
        file_path = os.path.join(directory_path, filename)
        if os.path.isfile(file_path):
            try:
                with open(file_path, "rb") as file:
                    byte_data = file.read()
                    try:
                        # Attempt to decode the bytes to a string
                        content = byte_data.decode(
                            "utf-8", errors="replace"
                        )  # Replace undecodable bytes
                    except Exception as decode_error:
                        content = f"[Unable to decode bytes: {decode_error}]"
                    print(f"\n=== {filename} ===")
                    print(content)
            except Exception as e:
                print(f"Failed to read file {filename}: {e}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python read_raw_files.py <directory_path>")
    else:
        read_and_print_files(sys.argv[1])
