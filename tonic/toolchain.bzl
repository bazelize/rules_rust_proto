def _tonic_toolchain_impl(ctx):
    return [platform_common.ToolchainInfo(
        runtime_deps = ctx.attr.runtime_deps,
        tonic_proto_transpiler = ctx.executable.tonic_proto_transpiler,
        protoc = ctx.executable.protoc,
    )]

tonic_toolchain = rule(
    implementation = _tonic_toolchain_impl,
    doc = "",
    attrs = {
        "runtime_deps": attr.label_list(),
        "tonic_proto_transpiler": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//tonic:tonic_proto_transpiler_bin"),
        ),
        "protoc": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@com_google_protobuf//:protoc"),
        ),
    },
)
