project(
  'hanami',
  'c',
  version: '0.1.0',
  license: 'MIT',
  default_options: [
    'c_std=c11',
    'warning_level=2',
    'werror=false',
  ],
)

add_project_arguments('-DWLR_USE_UNSTABLE', language: 'c')

wlroots = dependency('wlroots', version: '>=0.17.0')
wayland_server = dependency('wayland-server')
xkbcommon = dependency('xkbcommon')

wayland_scanner = find_program('wayland-scanner')

# xdg-shell (ต้อง generate เอง)
xdg_shell_xml = '/usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml'

xdg_shell_h = custom_target(
  'xdg-shell-protocol-h',
  input: xdg_shell_xml,
  output: 'xdg-shell-protocol.h',
  command: [wayland_scanner, 'server-header', '@INPUT@', '@OUTPUT@'],
)

xdg_shell_c = custom_target(
  'xdg-shell-protocol-c',
  input: xdg_shell_xml,
  output: 'xdg-shell-protocol.c',
  command: [wayland_scanner, 'private-code', '@INPUT@', '@OUTPUT@'],
)

# wlr-layer-shell (from wlroots source)
layer_shell_xml = '../wlroots/protocol/wlr-layer-shell-unstable-v1.xml'

layer_shell_h = custom_target(
  'wlr-layer-shell-protocol-h',
  input: layer_shell_xml,
  output: 'wlr-layer-shell-unstable-v1-protocol.h',
  command: [wayland_scanner, 'server-header', '@INPUT@', '@OUTPUT@'],
)

layer_shell_c = custom_target(
  'wlr-layer-shell-protocol-c',
  input: layer_shell_xml,
  output: 'wlr-layer-shell-unstable-v1-protocol.c',
  command: [wayland_scanner, 'private-code', '@INPUT@', '@OUTPUT@'],
)

executable(
  'hanami',
  [
    'main.c',
    xdg_shell_h,
    xdg_shell_c,
    layer_shell_h,
    layer_shell_c,
  ],
  dependencies: [
    wlroots,
    wayland_server,
    xkbcommon,
  ],
  install: true,
)
