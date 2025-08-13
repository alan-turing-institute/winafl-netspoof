import os
import random


def create_directory_and_files():
    # Directory name
    dir_name = "random_5_byte_inputs"

    # Create directory if it doesn't exist
    os.makedirs(dir_name, exist_ok=True)

    # Create 30 files
    for i in range(1, 31):
        # Generate 5 random bytes (each between 0 and 255)
        byte_values = [random.randint(0, 255) for _ in range(5)]

        # Print the bytes as a list of ints
        print(f"File {i}: {byte_values}")

        # Convert list of ints to bytes
        byte_data = bytes(byte_values)

        # Write the bytes to the file
        file_path = os.path.join(dir_name, f"file_{i}.bin")
        with open(file_path, "wb") as f:
            f.write(byte_data)


if __name__ == "__main__":
    create_directory_and_files()
