load("@rules_proto//proto:defs.bzl", "ProtoInfo")
load("@rules_rust//rust/private:utils.bzl", "determine_output_hash", "find_toolchain", "name_to_crate_name")
load("@rules_rust//rust/private:rustc.bzl", "rustc_compile_action")
load("@rules_rust//rust:defs.bzl", "rust_common")
load("@rules_rust//rust/private:providers.bzl", "BuildInfo", "CrateInfo", "DepInfo", "DepVariantInfo")
load("//toolchain:toolchain.bzl", "RUST_PROTO_TOOLCHAIN")

def _rust_proto_aspect_impl(target, ctx):
    proto_info = target[ProtoInfo]

    rust_proto_toolchain = ctx.toolchains[RUST_PROTO_TOOLCHAIN]
    rust_toolchain = find_toolchain(ctx)

    # Rust proto transpiler
    args = ctx.actions.args()
    inputs = []

    directory_name = "{}.rust".format(ctx.rule.attr.name)
    librs_file = ctx.actions.declare_file("{}/lib.rs".format(directory_name))
    cargo_toml_file = ctx.actions.declare_file("{}/Cargo.toml".format(directory_name))

    # Rust API
    args.add("--name", ctx.rule.attr.name)
    args.add("--edition", rust_toolchain.default_edition)
    args.add("--output_librs", librs_file.path)
    args.add("--output_cargo_toml", cargo_toml_file.path)

    # Proto API
    args.add_all("--direct_source", proto_info.direct_sources)
    args.add_all("--proto_path", proto_info.transitive_proto_path)
    args.add("--direct_descriptor_set", proto_info.direct_descriptor_set)
    args.add_all("--transitive_descriptor_set", proto_info.transitive_descriptor_sets)

    inputs.append(proto_info.direct_descriptor_set)
    inputs.extend(proto_info.transitive_sources.to_list())
    inputs.extend(proto_info.transitive_descriptor_sets.to_list())

    ctx.actions.run(
        executable = rust_proto_toolchain.proto_transpiler,
        arguments = [args],
        mnemonic = "RustProtoTranspiler",
        env = {
            "PROTOC": rust_proto_toolchain.protoc.path,
            "RUSTFMT": rust_toolchain.rustfmt.path,
        },
        inputs = inputs,
        tools = [rust_proto_toolchain.protoc, rust_toolchain.rustfmt],
        outputs = [
            librs_file,
            cargo_toml_file,
        ],
    )

    # Rust Library
    output_hash = determine_output_hash(librs_file)
    crate_name = name_to_crate_name(ctx.rule.attr.name)
    rust_lib = ctx.actions.declare_file("{}/lib{}-{}.rlib".format(
        cargo_toml_file.path,
        crate_name,
        output_hash,
    ))

    crate_dep_info = []
    for dep in ctx.rule.attr.deps + rust_proto_toolchain.runtime_deps:
        crate_info = dep[rust_common.crate_info]
        crate_dep_info.append(
            DepVariantInfo(
                crate_info = dep[CrateInfo] if CrateInfo in dep else None,
                dep_info = dep[DepInfo] if DepInfo in dep else None,
                build_info = dep[BuildInfo] if BuildInfo in dep else None,
                cc_info = dep[CcInfo] if CcInfo in dep else None,
            ),
        )

    rust_providers = rustc_compile_action(
        ctx = ctx,
        attr = ctx.rule.attr,
        toolchain = rust_toolchain,
        crate_info = rust_common.create_crate_info(
            name = crate_name,
            type = "rlib",
            root = librs_file,
            srcs = depset([librs_file]),
            deps = depset(crate_dep_info),
            proc_macro_deps = depset(crate_dep_info),
            output = rust_lib,
            aliases = {},
            edition = rust_toolchain.default_edition,
            rustc_env = {},
            is_test = False,
            compile_data = depset([]),
            owner = target.label,
        ),
        output_hash = output_hash,
    )

    # Drop the DefaultInfo because there can only be one DefaultInfo after the
    # aspect's providers have been merged with the rule's providers.
    return [provider for provider in rust_providers if type(provider) != "DefaultInfo"]

rust_proto_aspect = aspect(
    implementation = _rust_proto_aspect_impl,
    attr_aspects = ["deps"],
    doc = "The rust proto aspect which transpiles the direct and transitive proto_library definitions to Rust definitions.",
    attrs = {
        "_cc_toolchain": attr.label(
            doc = "The current cpp toolchain.",
            default = "@bazel_tools//tools/cpp:current_cc_toolchain",
        ),
        "_process_wrapper": attr.label(
            doc = "The process wrapper which is required by the Rust library rules.",
            default = Label("@rules_rust//util/process_wrapper"),
            executable = True,
            allow_single_file = True,
            cfg = "exec",
        ),
    },
    fragments = ["cpp"],
    toolchains = [
        "@rules_rust//rust:toolchain",
        "@bazel_tools//tools/cpp:toolchain_type",
        RUST_PROTO_TOOLCHAIN,
    ],
    incompatible_use_toolchain_transition = True,
)

def _rust_proto_library_impl(ctx):
    return [
        ctx.attr.proto[DefaultInfo],
        ctx.attr.proto[rust_common.crate_info],
        ctx.attr.proto[rust_common.dep_info],
    ]

rust_proto_library = rule(
    implementation = _rust_proto_library_impl,
    doc = "Rust proto library.",
    attrs = {
        "proto": attr.label(
            doc = "The proto_library to generate rust definitions.",
            providers = [ProtoInfo],
            aspects = [rust_proto_aspect],
            mandatory = True,
            cfg = "target",
        ),
    },
)
