use display_stream::{lazy_format, DisplayExt, JoinExt};

fn main() {
    let disp = (0..50)
        .map(|x| lazy_format!(move "`{x}`"))
        .lazy_join(", ")
        .omitted_with(
            60,
            |chars| lazy_format!(move "... (omitted {chars} characters)"),
        );
    println!("{disp}");
}
