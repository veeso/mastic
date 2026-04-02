# Cleans the build artifacts
[group('build')]
[confirm("Are you sure you want to clean the build artifacts?")]
clean:
  rm -rf {{WASM_DIR}}
  cargo clean

# Builds all canisters
[group('build')]
build_all: build_user build_directory build_federation

[group('build')]
build_directory: pre_build
  just build_canister "directory" "directory"

[group('build')]
build_federation: pre_build
  just build_canister "federation" "federation"

[group('build')]
build_user: pre_build
  just build_canister "user" "user"

[private]
pre_build: 
  mkdir -p "{{WASM_DIR}}"

[private]
build_canister canister_name wasm_name:
  echo "Building {{canister_name}} Canister"
  cargo build --target wasm32-unknown-unknown --release --package "{{canister_name}}"
  ic-wasm "target/wasm32-unknown-unknown/release/{{canister_name}}.wasm" -o "{{WASM_DIR}}/{{wasm_name}}.wasm" shrink
  candid-extractor "{{WASM_DIR}}/{{wasm_name}}.wasm" > "{{WASM_DIR}}/{{wasm_name}}.did"
  gzip -k "{{WASM_DIR}}/{{wasm_name}}.wasm" --force
