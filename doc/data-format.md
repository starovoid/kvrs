# Data format
The data file contains binary values and by default has the **.kv** extension.

| Section                     | Type   | Size (in bytes) |
| --------------------------- | ------ | --------------- |
| identifier                  | `u64`  | 8               |
| version                     | `u8`   | 1               |
| index position              | `u64`  | 8               |
| vacant blocks list position | `u64`  | 8               |
| record 1                    | `[u8]` | arbitary        |
| ...                         | ...    | ...             |
| record n                    | `[u8]` | arbitary        |
| vacant blocks list          | `[u8]` | arbitary        |

---

- **Identifier**

    The identifier is a "magic" number so that using it the program can make sure that it is actually dealing with a file of the correct format.

- **Version**

    Just the file format version.

- **Index position**

    The byte number from which begins record containing the index.

- **Vacant blocks list position**

    The byte number from which the list of vacant blocks begins.


## Record format
| Section            | Type   | Size (in bytes) |
| -------            | ------ | --------------- |
| data len (size)    | `u64`  | 8               |
| data               | `[u8]` | size            |


## Vacant blocks list format
Sorted `Vec<VacantBlock>` serialized in [postcard](https://crates.io/crates/postcard) format.

```rust
struct VacantBlock {
    pos: u64,   // It's position in data file
    size: u64,  // Size in bytes
}
```
