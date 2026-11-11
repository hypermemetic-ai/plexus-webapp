//! The README's Rust blocks are this toy's files.
//!
//! A fenced block under a comment that names a file, `<!-- toy/build.rs -->`,
//! is lines that stand together in that file. A block that differs from its
//! file fails this test; so does a `rust` block that names no file.

use std::fs;
use std::path::Path;

#[test]
fn every_rust_block_is_its_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let text = fs::read_to_string(root.join("README.md")).unwrap();
    let readme: Vec<&str> = text.lines().collect();

    let mut checked = 0;
    let mut index = 0;
    while index < readme.len() {
        if !readme[index].starts_with("```") {
            index += 1;
            continue;
        }
        let close = (index + 1..readme.len()).find(|&i| readme[i].starts_with("```")).expect("the block is closed");
        let source = index.checked_sub(1).and_then(|i| readme[i].strip_prefix("<!-- ")).and_then(|rest| rest.strip_suffix(" -->"));
        match source {
            Some(source) => {
                let file = fs::read_to_string(root.join(source)).unwrap_or_else(|e| panic!("{source}: {e}"));
                let file: Vec<&str> = file.lines().map(str::trim_end).collect();
                let block = &readme[index + 1..close];
                assert!(
                    file.windows(block.len()).any(|window| window == block),
                    "README.md:{}: not what {source} says",
                    index + 2
                );
                checked += 1;
            }
            None => assert_ne!(readme[index], "```rust", "README.md:{}: a rust block names no file", index + 1),
        }
        index = close + 1;
    }
    assert!(checked > 0, "README.md names no file");
}
