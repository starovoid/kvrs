# About
Console binary key-value storage with output highlighting, searchability with regular expressions and aliasing of bash commands.

# Usage
## Basic operations
### Set a key-value pair:
```bash
kvrs set "key" "value" --file="storage.kv"
```

### Get the value by key
```bash
kvrs get "key" --file="storage.kv"
```

### Update the value by key
```bash
kvrs update "key" "new value" --file="storage.kv"
```

### Delete a key-value pair
```bash
kvrs remove "key" --file="storage.kv"
```

### Find a key-value pair by regexp
```bash
kvrs find "regexp" --file="storage.kv"
```
This command tries to treat the keys as text and match them to the sample, returning all matches.

### Sort
```bash
kvrs sort --file="storage.kv"
```
Returns a list of keys in lexicographic order.

### Run the value as a bash command
```bash
kvrs set "list" "ls -la"
kvrs cmd "list" --file="storage.kv"
```

## Common parameters
- ```--file``` - the database file that the command will work with. By default, this parameter will be equal to the path to the file "storage.kv" in the home directory.

# Roadmap
- [ ] get, set, update, remove
- [ ] cmd
- [ ] sort
- [ ] find
