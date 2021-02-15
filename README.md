# lab_proto_gen

```
git clone git@github.com:coingaming/lab_proto_gen.git -b grpc
cd lab_proto_gen
cargo build --release
cd [coingaming_flask_repo_dir]/protos
[lab_proto_gen_repo_dir]/target/release/lab_proto_gen
(cd lab_protobuf && git add . && git commit -am "grpc gen" && git push)
(cd coingaming_protobuf && git add . && git commit -am "grpc gen" && git push)
git commit -am "grpc gen" && git push
```
