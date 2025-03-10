# SPDX-FileCopyrightText: d-k-bo <d-k-bo@mailbox.org>
# SPDX-License-Identifier: CC0-1.0

desktop_file = i18n.merge_file(
  input: BASE_APP_ID + '.desktop.in',
  output: BASE_APP_ID + '.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
  input: BASE_APP_ID + '.metainfo.xml.in',
  output: BASE_APP_ID + '.metainfo.xml',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

# Setup a translated metainfo file in the resources dir as well
# for use by the about dialog
# based on https://github.com/bragefuglseth/fretboard/blob/cf9e72bf66dfc4fb0b24ba8cf04c7f794191dbc8/data/meson.build#L41-L50
msgfmt = find_program('msgfmt', required: true)
run_command(
  msgfmt,
  '--xml',
  '-d' + meson.project_source_root() / 'po',
  '--template=' + BASE_APP_ID + '.metainfo.xml.in',
  '--output-file=' + 'resources' / BASE_APP_ID + '.metainfo.xml',
  check: true,
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
if appstreamcli.found()
  test(
    'Validate appstream file',
    appstreamcli,
    args: ['validate', '--no-net', '--explain', appstream_file],
  )
endif

install_data(
  BASE_APP_ID + '.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)
install_data(
  BASE_APP_ID + '.Player' + '.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

gnome.compile_schemas(depend_files: meson.current_source_dir())

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
if compile_schemas.found()
  test(
    'Validate schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()],
  )
endif

subdir('icons')