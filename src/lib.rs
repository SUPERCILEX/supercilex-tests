#![feature(exit_status_error)]
#![allow(clippy::missing_panics_doc)]

use std::{env, fmt::Write, path::PathBuf, process};

use expect_test::expect_file;
use public_api::PublicApi;

pub fn fmt() {
    #[cfg(not(miri))]
    {
        let mut command = process::Command::new("cargo");
        command.args(["fmt", "--all"]);

        let config = "--config=version=Two,imports_granularity=Crate,\
                      group_imports=StdExternalCrate,use_field_init_shorthand=true,\
                      format_code_in_doc_comments=true,format_macro_bodies=true,\
                      format_macro_matchers=true,format_strings=true,wrap_comments=true";
        if env::var_os("UPDATE_EXPECT").is_some() {
            command
                .args(["--", config])
                .status()
                .unwrap()
                .exit_ok()
                .unwrap();
        } else {
            command
                .args(["--", "--check", config])
                .status()
                .unwrap()
                .exit_ok()
                .unwrap();
        }
    }
}

pub fn clippy() {
    #[cfg(not(miri))]
    {
        let mut command = process::Command::new("cargo");
        command.args(["clippy", "--workspace", "--all-targets", "--all-features"]);

        let config = [
            "-W",
            "clippy::all",
            "-W",
            "clippy::pedantic",
            "-W",
            "clippy::nursery",
            "-W",
            "clippy::cargo",
            "-W",
            "clippy::float_cmp_const",
            "-W",
            "clippy::empty_structs_with_brackets",
            "-A",
            "clippy::multiple_crate_versions",
        ];
        if env::var_os("UPDATE_EXPECT").is_some() {
            command
                .args(["--fix", "--allow-dirty", "--allow-staged", "--"])
                .args(config)
                .status()
                .unwrap()
                .exit_ok()
                .unwrap();
        } else {
            command
                .args(["--", "-D", "warnings"])
                .args(config)
                .status()
                .unwrap()
                .exit_ok()
                .unwrap();
        }
    }
}

pub fn api() {
    #[cfg(not(miri))] // gnu_get_libc_version breaks miri
    {
        let json_path = rustdoc_json::Builder::default()
            .all_features(true)
            .build()
            .unwrap();

        let api = PublicApi::from_rustdoc_json(json_path, public_api::Options::default())
            .unwrap()
            .to_string();

        expect_file![path_from_root("api.golden")].assert_eq(&api);
    }
}

pub fn help_for_review(mut command: clap::Command) {
    #[cfg(not(miri))] // wrap_help breaks miri
    {
        command.build();

        let mut long = String::new();
        let mut short = String::new();

        write_help(&mut long, &mut command, LongOrShortHelp::Long);
        write_help(&mut short, &mut command, LongOrShortHelp::Short);

        expect_file![path_from_root("command-reference.golden")].assert_eq(&long);
        expect_file![path_from_root("command-reference-short.golden")].assert_eq(&short);
    }
}

fn path_from_root(path: &str) -> PathBuf {
    let mut dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    dir.push(path);
    dir
}

#[derive(Copy, Clone)]
enum LongOrShortHelp {
    Long,
    Short,
}

fn write_help(
    buffer: &mut impl Write,
    cmd: &mut clap::Command,
    long_or_short_help: LongOrShortHelp,
) {
    write!(
        buffer,
        "{}",
        match long_or_short_help {
            LongOrShortHelp::Long => cmd.render_long_help(),
            LongOrShortHelp::Short => cmd.render_help(),
        }
    )
    .unwrap();

    for sub in cmd.get_subcommands_mut() {
        writeln!(buffer).unwrap();
        writeln!(buffer, "---").unwrap();
        writeln!(buffer).unwrap();

        write_help(buffer, sub, long_or_short_help);
    }
}
