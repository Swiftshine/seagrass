# Seagrass
(i'll write details on what this actually is later since this is still a WIP)


## Language Implementation
### Tags
Tags modify how the language, and certain elements, may be interpreted. Tags that are applied to things such as structs and functions are called attributes. Tags that might be used include:

```
#[pod] // used on a struct to indicate that it can be read and written to using a stream of bytes
#[run_on_import] // used on a function to indicate that it will run if and only if the script file that owns it gets imported into another; it will raise an error if called manually

#[configure_runtime("preserve_expired_frames", true)] // indicates that the runtime should keep track of expired function frames for inspection after the runtime's execution has ended

#[configure_interpreter("enforce_explicit_type_annotation", true)] // self-explanatory
```


### Structs
Serializable structs must be marked as POD with the `#[pod]` tag, as a promise that the struct has a stable memory layout.
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

*All* fields within a serializable POD struct must be serializable themselves, including sub-structs.

### References
Functions must not return references.
