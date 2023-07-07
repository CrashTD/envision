subdir('icons')
# Desktop file
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: global_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications'
)
# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file.full_path()
    ],
    depends: desktop_file,
  )
endif

# Appdata
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: global_conf
  ),
  output: '@0@.appdata.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'metainfo'
)
# Validate Appdata
if appstream_util.found()
  test(
    'validate-appdata', appstream_util,
    args: [
      'validate', '--nonet', appdata_file.full_path()
    ],
    depends: appdata_file,
  )
endif

#### Gschema currently disabled, using json config
# GSchema
# configure_file(
#   input: '@0@.gschema.xml.in'.format(base_id),
#   output: '@0@.gschema.xml'.format(application_id),
#   configuration: global_conf,
#   install: true,
#   install_dir: datadir / 'glib-2.0' / 'schemas'
# )

# Validata GSchema
# if glib_compile_schemas.found()
#   test(
#     'validate-gschema', glib_compile_schemas,
#     args: [
#       '--strict', '--dry-run', meson.current_build_dir()
#     ],
#   )
# endif

resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)
