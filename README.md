## rules_tonic

Bazel rules for compiling grpc and protobufs using Tonic and Prost.

## Using


Load the rules by adding the following in your `WORKSPACE`:
````python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_tonic",
    sha256 = "<sha256>",
    urls = ["https://github.com/bazelwork/rules_tonic/archive/refs/heads/main.zip"],
)
````

In your `BUILD.bazel` add:
````python
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("//:defs.bzl", "rust_proto_library")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "echo_proto",
    srcs = [
        "echo.proto",
        "echo.messages.proto",
    ],
    deps = []
)

rust_proto_library(
    name = "echo_proto.rs",
    proto = ":echo_proto",
)

rust_binary(
    name = "echo",
    srcs = ["src/main.rs"],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(proc_macro = True),
    deps = all_crate_deps(normal = True) + [
        ":echo_proto.rs",
    ],
)
````

See the `examples/` directory for more use cases.

## Recommendations

We recommend using the [`cargo-bazel` rules](https://github.com/abrisco/cargo-bazel/) for 
managing Rust cargo dependencies.
