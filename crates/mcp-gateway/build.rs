fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the path to the proto file
    let proto_file = "../../proto/mcp.proto";
    
    println!("cargo:rerun-if-changed={}", proto_file);
    println!("cargo:rerun-if-changed=../../proto");
    
    // In development environment, only display a warning and skip if protoc is not found
    if let Err(e) = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile(&[proto_file], &["../../proto"])
    {
        println!("cargo:warning=Failed to compile protobufs: {}", e);
        println!("cargo:warning=Continuing without recompiling protobufs...");
        println!("cargo:warning=This is acceptable for development, but should be fixed for production.");
    }
    
    Ok(())
} 