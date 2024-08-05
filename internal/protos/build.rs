fn main() {
    let out_dir = format!("{}/prom_protos", std::env::var("OUT_DIR").unwrap());

    std::fs::create_dir_all(&out_dir).unwrap();
    prost_build::Config::new()
        .out_dir(out_dir)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&[
            "prom_protos/message.proto",
        ], &["prom_protos"])
        .unwrap();

    let out_dir = "../../target".to_string();
    let builder = tonic_build::configure();
    builder.out_dir(out_dir.clone())
        .compile(&["prom_protos/message.proto"], &["prom_protos"])
        .unwrap();
}