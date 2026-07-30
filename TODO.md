include statements

- i'd also like some functions to run only when they're configured to upon load, perhaps to initialise values or something

vectors

- dependent-length arrays might look something like this:

```rs
struct MyStruct {
    count: u32,
    #[counted_by(count)]
    things: [u32]
}
```

enums

- pure, numerical-only enums
- enums that contain data based on values

REAL ERROR PARSING!!!!!!! right now it's ambiguous as hell

utility functions perhaps

- crc32, md5, sha256, the like. requires that a data type be serializable

make file/io function tests (make them return a vec instead of a runtime value)

reading raw bytes

byte streams

hex dumps

string formatting function (like format!(...))

filesystem functions

assertions

write tests for like. every operator ever

actual unary operations (negation)
