config = configure_file(
  input: 'constants.rs.in',
  output: 'constants.rs',
  configuration: global_conf
)
# Copy the constants.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'constants.rs',
  meson.project_source_root() / 'src' / 'constants.rs',
  check: true
)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('debugbuild')
  rust_target = 'debug'
  message('Building in debug mode')
else
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
endif

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)

test(
  'cargo-fmt-check',
  cargo,
  args: ['fmt', '--all', '--check']
)

test(
  'cargo-clippy',
  cargo,
  env: [],
  args: ['clippy', '--all-targets', '--all-features', '--', '-Dwarnings'],
  timeout: 0,
)

test(
  'cargo-test',
  cargo,
  args: ['test'],
  timeout: 0,
)
