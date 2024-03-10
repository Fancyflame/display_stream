# Display Stream

Makes display be a lazy stream, to avoid unnecessary heap allocation and supports `no_std`.

```rust
let disp = (0..50)
    .map(|x| lazy_format!(move "`{x}`"))
    .lazy_join(", ")
    .omitted_with(
        60,
        |chars| lazy_format!(move "... (omitted {chars} characters)"),
    );
println!("{disp}");
```

prints

```
`0`, `1`, `2`, `3`, `4`, `5`, `6`, `7`, `8`, `9`, `10`, `11`... (omitted 228 characters)
```