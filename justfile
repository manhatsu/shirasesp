build:
    cargo fmt --all
    cargo clippy --target xtensa-esp32-none-elf -- -D warnings
    cargo build --release

flash: build
    espflash flash --monitor target/xtensa-esp32-none-elf/release/shirasesp --chip esp32 --log-format defmt

monitor:
    espflash monitor --chip esp32 --log-format defmt
