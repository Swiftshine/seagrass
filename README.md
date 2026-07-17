# Seagrass
(i'll write details on what this actually is later)


## Language Implementation
### Structs
Serializable structs must be marked with `#[pod]`:
```
// this one cannot be used to write to disk directly
struct StructOne {
    field_1: u32
}

// this one CAN be used to write to disk directly
#[pod]
struct StructTwo {
    field_1: u32
}
```

*All* fields within a serializable POD struct must be serializable themselves, including sub-structs.

### References
Functions must not return references.
