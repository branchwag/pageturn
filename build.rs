use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let posts_dir = Path::new("posts");

    // Create posts directory if it doesn't exist
    if !posts_dir.exists() {
        fs::create_dir(posts_dir).expect("Failed to create posts directory");
    }

    // Tell cargo to rerun this build script if the posts directory changes
    println!("cargo:rerun-if-changed=posts");

    // Generate list of markdown files
    let mut post_files = Vec::new();

    if let Ok(entries) = fs::read_dir(posts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                post_files.push(file_name.to_string());
            }
        }
    }

    post_files.sort();

    // Generate Rust code with correct relative paths
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_posts.rs");
    let mut f = fs::File::create(&dest_path).unwrap();
    writeln!(f, "pub const POST_FILES: &[&str] = &[").unwrap();
    for file in &post_files {
        write!(
            f,
            "    include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/posts/{}\")),",
            file
        )
        .unwrap();
        writeln!(f).unwrap();
    }
    writeln!(f, "];").unwrap();

    println!("Generated includes for {} markdown files", post_files.len());
}
