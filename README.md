# Seagrass

A scripting language focused on ease of file I/O for binary file formats and filesystem utility.

Demo:

```rs
#[pod]
struct Header {
    magic: u32,
    version: u16,
    flags: u16,
    data: u32
}

let handle = sg::open("file.bin");

let header = handle.read<Header>();

header.data = 123;

sg::write("file_2.bin", &header);

handle.close();
handle.delete();
```

## Installation

Not available yet! Seagrass is still being worked on.

## Language Details

### Tags

Tags modify how the language, and certain elements, may be interpreted. Tags that are applied to things such as structs and functions are called attributes. Tags that might be used include:

```rs
#[pod] // used on a struct to indicate that it can be read and written to using a stream of bytes
#[byte_order("big")] // self-explanatory
```

### Structs

Serializable structs must be marked as POD with the `#[pod]` attribute, as a promise that the struct has a stable memory layout **upon serialization**.

```rs
// this one cannot be serialized
struct StructOne {
    field_1: u32
}

// this one CAN be serialized
#[pod]
struct StructTwo {
    field_1: u32
}
```

_All_ fields within a serializable POD struct must be serializable themselves, including sub-structs.

#### Notes on Serializability

The `string` data type is _not_ serializable by default _unless_ it's marked with the `#[serialize_as("...")]` attribute. Note that strings tagged with this will be null-terminated. Example:

```rs
#[pod]
struct MyStruct {
    #[serialize_as("ascii")] // null-terminated
    field: string
}
```

Padding between data types must always be explicit. When serializing, the value used to fill in gaps is `0x00`.

```rs
#[pod]
struct MyStruct {
    field_1: u8,
    pad(3),
    field_2: u32
}
```

Alignment is similar to padding, but is used for fields that come after serializable data types whose sizes can only be determined at runtime, specifying when said field should begin.

```rs
#[pod]
struct MyStruct {
    field_1: u32,
    #[serialize_as("ascii")]
    field_2: string,
    #[align(4)] // "this field will be serialized on a byte boundary of 4"
    field_3: u32
}
```

### Type Notes

Seagrass' type casting syntax is the same as Rust's. It also supports type coercion.

Floating-point values **must** be postfixed with an `f` or `d` depdending on the type.

```rs
let a: f32 = 1.0f;
let b: f64 = 1.0d;

// not permitted due to ambiguity
let c = 1.0;
```

## Notes

Q: What is Seagrass?

A: Seagrass is a personal project and a work in progress; anything listed here might be subject to change or absent entirely.

Q: Why does it look like Rust? Does it work the same way?

A: Seagrass' syntax is based on Rust's (and is written in Rust), but that's it. It doesn't have the ownership or memory rules associated with Rust and its compiler because Seagrass, an interpreted language for inherently "quick and dirty" operations, is not meant for that.

Q: Why make a scripting language for this?

A: Because there aren't any existing _interpreted_ languages that do this well in any capacity.
