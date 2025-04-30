use std::path::PathBuf;
use std::fs::create_dir;
use std::fs::read_dir;
use std::fs::copy;
use dunce::canonicalize;

const OPAQUE_TYPES: &[&str] = &["nam::Buffer", "std::.*", "Eigen::.*", "__gnu_cxx::.*", "nlohmann::.*"];
const BLOCK_TYPES: &[&str] = &["pointer", "size_type", "difference_type", "const_pointer", "value_type"];


fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.cpp");
    println!("cargo:rerun-if-changed=Cargo.lock");

    // This is the directory where the `c` library is located.
    let libdir_path = canonicalize(PathBuf::from("NeuralAmpModelerCore/")).expect("Couldn't canonicalize path");
    let build_path = libdir_path.join("build");
    if let Err(e) = create_dir(&build_path) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("could not create build directory: {}", e);
        }
    }

    // This is the path to the `c` headers file.
    let headers_path = libdir_path.join("NAM/wrapper.h");
    let impl_path = libdir_path.join("NAM/wrapper.cpp");
    let headers_path_str = headers_path.to_str().unwrap(); 

    // Copy header.h to NAM directory
    copy("wrapper.h", &headers_path).expect("Couldn't copy wrapper.h file");
    // Copy wrapper.cpp to NAM directory
    copy("wrapper.cpp", &impl_path).expect("Couldn't copy wrapper.cpp file");

    // Compile the cpp files in the `libdir_path` directory
    let allowlist = read_dir(libdir_path.join("NAM")).expect("Couldn't read directory").filter_map(|entry| {
        let entry = entry.expect("Couldn't get entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
            Some(path)
        } else {
            None
        }
    }).collect::<Vec<_>>();
 
    // Run cc
    cc::Build::new()
        .files(allowlist.iter())
        .cpp(true)
        .include(libdir_path.join("Dependencies/nlohmann").to_str().unwrap())
        .include(libdir_path.join("Dependencies/eigen").to_str().unwrap())
        .include(libdir_path.join("NAM").to_str().unwrap())
        .std("c++20")
        .out_dir(libdir_path.join("build"))
        .compile("nam_core_static");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(headers_path_str)
        .respect_cxx_access_specs(true)
        .clang_arg("-x").clang_arg("c++")
        .clang_arg("-std=c++20")
        .clang_arg("-I").clang_arg(libdir_path.join("Dependencies/nlohmann").to_str().unwrap())
        .clang_arg("-I").clang_arg(libdir_path.join("Dependencies/eigen").to_str().unwrap());

    for entry in allowlist {
        // Regex that matches the file name of the header file
        builder = builder.allowlist_file(format!(".*{}\\.h", entry.file_stem().unwrap().to_str().unwrap()));
    }

    for opaque in OPAQUE_TYPES {
        builder = builder.opaque_type(opaque);

    }

    for block in BLOCK_TYPES {
        builder = builder.blocklist_type(block);
    }

    let bindings = builder
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}