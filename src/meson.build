configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: CONFIG
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

help_overlay = custom_target(
  'help-overlay',
  input: files('help-overlay.blp'),
  output: '.',
  command: [
    find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@', '&&',
    'mkdir', '-p', meson.project_source_root() / 'data/resources/gtk/', '&&',
    'mv', meson.project_build_root() / 'src' /'help-overlay.ui', meson.project_source_root() / 'data/resources/gtk/'
  ],
)

fs = import('fs')

cargo = find_program('cargo')
cargo_opt = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_env = {}

if fs.exists('/.flatpak-info')
  cargo_env += { 'CARGO_HOME': meson.project_build_root() / 'cargo-home' }
endif

if get_option('profile') == 'release'
  cargo_opt += [ '--release' ]
  rust_target = 'release'
else
  rust_target = 'debug'
endif

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  env: cargo_env,
  command: [
    cargo, 'build', cargo_opt, '&&',
    'cp', meson.project_source_root() / 'target' / rust_target / meson.project_name(), '@OUTPUT@',
  ],
  depends: help_overlay,
)