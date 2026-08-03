# Native Functions for Seagrass

## Output

- `sg::set_byte_order(byte_order: string)`
  - Used when serializing data types or values that cannot have a data type annotated with the `#[byte_order(...)]` attribute.
  - `byte_order` must be `"big"` or `"little"`

```rs
sg::set_byte_order("big");
// or
sg::set_byte_order("little");
```

- `sg::print<auto T>(printable: T)`
  - Used when printing a single value to the console. The data type is also displayed.

```rs
sg::print("Hello, world!");
sg::print(123);
```

## Filesystem

- `sg::open_file(filepath: string) -> sg::FileHandle`
  - Creates an already opened file handle.

- `sg::read<T>(filename: string) -> T` where `T` is a serializable data type.

```rs
let value = sg::read<SerializableStructType>("input.bin");
```

- `sg::write<auto T>(filename: string, data: T)` where `T` is a serializable data type. Note that `T` is auto-detected.

```rs
let value = ...;
sg::write("output.bin", &value);
// or
sg::write("output.bin", value);
```
