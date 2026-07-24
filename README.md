# Seagrass

(i'll write details on what this actually is later since this is still a WIP)

## Language Implementation

### Tags

Tags modify how the language, and certain elements, may be interpreted. Tags that are applied to things such as structs and functions are called attributes. Tags that might be used include:

```
#[pod] // used on a struct to indicate that it can be read and written to using a stream of bytes
#[byte_order("big")] // self-explanatory

#[run_on_import] // used on a function to indicate that it will run if and only if the script file that owns it gets imported into another; it will raise an error if called manually

#[configure_runtime("preserve_expired_frames", true)] // indicates that the runtime should keep track of expired function frames for inspection after the runtime's execution has ended

#[configure_interpreter("enforce_explicit_type_annotation", true)] // self-explanatory
```

### Structs

Serializable structs must be marked as POD with the `#[pod]` attribute, as a promise that the struct has a stable memory layout.

```
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

The `string` data type is _not_ serializable by default _unless_ it's marked with the `#[serialize_as("...")]` attribute. Example:

```
#[pod]
struct MyStruct {
    #[serialize_as("ascii")] // null-terminated
    field: string
}
```

Padding between data types must always be explicit. When serializing, the value used to fill in gaps is `0x00`.

```
#[pod]
struct MyStruct {
    field_1: u8,
    padding(3),
    field_2: u32
}
```

Alignment is similar to padding, but is used for serializable data types whose sizes can only be determined at runtime, specifying when the next field should begin.

```
#[pod]
struct MyStruct {
    field_1: u32,
    field_2: string,
    align(4),
    field_3: u32
}
```

### References

Functions must not return references.
