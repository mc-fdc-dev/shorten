cargo build --release
cp ./target/release/tinyurl ./tinyurl
zip -r release.zip ./tinyurl
rm ./tinyurl