cargo build --release
cp ./target/release/tinyurl ./tinyurl
zip release.zip ./tinyurl
rm ./tinyurl