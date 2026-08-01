# `sg::FileHandle`

- `.close()`
    - Closes the file handle.

- `.delete()`
    - Deletes the file. The file handle must be closed first.

- `.read<auto T>()`
    - Reads a whole `T` in one go.

- `.read_value<auto T>()`
    - Reads a `T`, but progresses the stream position.

- `.rename(new_path: string)`
    - Renames the file to this path. The file handle must be closed first.
