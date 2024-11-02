project(
  'envision',
  'rust',
  version: '0.0.2', # version number row
  meson_version: '>= 0.59',
  license: 'AGPL-3.0',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'org.gabmus.envision'
pretty_name = 'Envision'
upstream_repo = 'https://gitlab.com/gabmus/envision'
author = 'The Envision Team'
description = 'Orchestrator for the free XR stack' # temporary

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.10.0')
dependency('vte-2.91-gtk4', version: '>= 0.72.0')

glib_compile_resources = find_program('glib-compile-resources', required: true)
# glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
cargo = find_program('cargo', required: true)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

# are we building a tagged version?
if run_command('git', 'describe', '--tags', '--exact-match').returncode() != 0
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  version_suffix = ''
  application_id = base_id
endif

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name()
  + '-'
  + version,
  meson.project_source_root(),
)

global_conf = configuration_data()
global_conf.set('APP_ID', application_id)
global_conf.set('RESOURCES_BASE_PATH', '/' + base_id.replace('.', '/'))
global_conf.set('PKGDATADIR', pkgdatadir)
global_conf.set('PROFILE', profile)
global_conf.set('VERSION', version + version_suffix)
global_conf.set('GETTEXT_PACKAGE', gettext_package)
global_conf.set('LOCALEDIR', localedir)
global_conf.set('AUTHOR', author)
global_conf.set('PRETTY_NAME', pretty_name)
global_conf.set('CMD_NAME', meson.project_name())
global_conf.set('REPO_URL', upstream_repo)
global_conf.set('ISSUES_URL', upstream_repo + '/-/issues')
global_conf.set(
  'BUILD_DATETIME',
  run_command('date', '-Iseconds', '--utc', check: true).stdout().strip(),
)

subdir('data')
subdir('po')
subdir('src')
subdir('scripts')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: false,
  update_desktop_database: true,
)
