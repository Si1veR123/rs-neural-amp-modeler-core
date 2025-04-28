use std::path::PathBuf;
use std::fs::create_dir;
use std::fs::read_dir;
use std::fs::copy;
use dunce::canonicalize;

const OPAQUE_TYPES: &[&str] = &["nam::Buffer", "std::.*", "Eigen::.*", "__gnu_cxx::.*", "nlohmann::.*"];
const BLOCK_TYPES: &[&str] = &["pointer", "size_type", "difference_type", "const_pointer", "value_type"];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=CMakeLists.txt");


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

    // Generate bindings for the headers in the `libdir_path` directory
    let allowlist = read_dir(libdir_path.join("NAM")).expect("Couldn't read directory").filter_map(|entry| {
        let entry = entry.expect("Couldn't get entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("h") {
            Some(path)
        } else {
            None
        }
    }).collect::<Vec<_>>();

    // Copy correct CMakeLists.txt file to libdir_path directory
    let cmake_path = libdir_path.join("CMakeLists.txt");
    copy("CMakeLists.txt", &cmake_path).expect("Couldn't copy CMakeLists.txt file");

    // This is the path to the static library file.
    println!("cargo:rustc-link-search=native={}", build_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=nam_core_static");

    // Run cmake
    if !std::process::Command::new("cmake")
        .arg("-S")
        .arg(&libdir_path)
        .arg("-B")
        .arg(&build_path)
        .status()
        .expect("could not spawn `cmake`")
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not run cmake");
    }

    // Run make/mingw32-make
    if let Err(_) = std::process::Command::new("make")
        .arg("-C")
        .arg(&build_path)
        .status()
    {
        if !std::process::Command::new("mingw32-make")
            .arg("-C")
            .arg(&build_path)
            .status()
            .expect("could not spawn `make/mingw32-make`")
            .success() {
            // Panic if the command was not successful.
            panic!("could not run make/mingw32-make");
        }
    }

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
        .clang_arg("-stdlib=libstdc++")
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