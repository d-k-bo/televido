# SPDX-FileCopyrightText: d-k-bo <d-k-bo@mailbox.org>
# SPDX-License-Identifier: CC0-1.0

scalable_dir = 'hicolor' / 'scalable' / 'apps'
install_data(
  scalable_dir / ('@0@.svg').format(APP_ID),
  install_dir: get_option('datadir') / 'icons' / scalable_dir
)

symbolic_dir = 'hicolor' / 'symbolic' / 'apps'
install_data(
  symbolic_dir / ('@0@-symbolic.svg').format(BASE_APP_ID),
  install_dir: get_option('datadir') / 'icons' / symbolic_dir
)