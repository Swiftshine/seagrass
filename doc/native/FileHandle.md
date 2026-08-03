# `sg::FileHandle`

- `.close()`
  - Closes the file handle.

- `.delete()`
  - Deletes the file. The file handle must be closed first.

- `new(file_path: string)`
  - Creates a closed file handle to `file_path`.

- `.open()`
  - Opens the file handle.
  - The effects of `FileHandle::new()` and `FileHandle::.open()` can be achieved in one function call through `sg::open_file()`.

- `.read<auto T>()`
  - Reads a whole `T` in one go.

- `.read_value<auto T>()`
  - Reads a `T`, but progresses the stream position.

- `.rename(new_path: string)`
  - Renames the file to this path. The file handle must be closed first.
