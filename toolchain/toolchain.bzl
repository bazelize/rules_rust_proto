"""Rust Protobuf Toolchain definitions."""

RUST_PROTO_TOOLCHAIN = str(Label("//toolchain:toolchain_type"))

def _rust_proto_toolchain_impl(ctx):
    return [platform_common.ToolchainInfo(
        runtime_deps = ctx.attr.runtime_deps,
        proto_transpiler = ctx.executable.proto_transpiler,
        protoc = ctx.executable.protoc,
    )]

rust_proto_toolchain = rule(
    implementation = _rust_proto_toolchain_impl,
    doc = """Rust protobuf toolchain.
    
    TODO: add details on the CLI API spec.
    """,
    attrs = {
        "runtime_deps": attr.label_list(
            doc = "The rust dependencies required at execution runtime.",
        ),
        "proto_transpiler": attr.label(
            doc = "The rust proto transpiler binary.",
            executable = True,
            cfg = "exec",
        ),
        "protoc": attr.label(
            doc = "The protoc binary.",
            executable = True,
            cfg = "exec",
            default = Label("@com_google_protobuf//:protoc"),
        ),
    },
)
