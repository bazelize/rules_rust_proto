workspace(name = "rules_tonic")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# To get an accurate and up to date repository definition, See the releases page
# https://github.com/abrisco/cargo-bazel/releases
http_archive(
    name = "cargo_bazel",
    # sha256 = "{sha256}",
    urls = ["https://github.com/abrisco/cargo-bazel/releases/download/0.0.11/cargo_bazel.tar.gz"],
)

load("@cargo_bazel//:deps.bzl", "cargo_bazel_deps")

cargo_bazel_deps()

# It's important to set a constant for desired Rust version so it
# can easily be passed to each `crates_repository` definition.
RUST_VERSION = "1.56.0"

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(version = RUST_VERSION)

# Cargo dependencies
load("@cargo_bazel//:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    lockfile = "//:Cargo.bazel.lock",
    manifests = ["//:Cargo.toml"],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

http_archive(
    name = "com_google_protobuf",
    sha256 = "87407cd28e7a9c95d9f61a098a53cf031109d451a7763e7dd1253abf8b4df422",
    strip_prefix = "protobuf-3.19.1",
    urls = ["https://github.com/protocolbuffers/protobuf/archive/refs/tags/v3.19.1.tar.gz"],
)

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

register_toolchains(
    "//tonic:toolchain",
)
