#!/usr/bin/bash

set -ex

OPCBINARIES=(opcimginstall opcimguninstall opcimgattach opcimgdetach opcimgstatechange opcimgquery opcimgbuild)
BINARY_INSTALL=(install uninstall attach detach statechange query build_runner)
BRANDS=(opczimage opcnative opcpropolis opcbhyve)
BUILD_DIR="${HOME}/.cargo/target/debug"
BRAND_DIR="/usr/lib/brand"
BRAND_CONFIG_FILES=(config.xml platform.xml)

for bin in ${OPCBINARIES[@]}; do
  cargo build --package opczone --bin $bin
done

cargo build --package imgbuild --bin imgbuild

if [ "$1" == "--install" ]; then
  for brand in ${BRANDS[@]}; do
    sudo mkdir -p "${BRAND_DIR}/${brand}"
    for idx in ${!OPCBINARIES[*]}; do
      sudo ln -sf "${BUILD_DIR}/${OPCBINARIES[$idx]}" "${BRAND_DIR}/${brand}/${BINARY_INSTALL[$idx]}"
    done
    for cfg_file in ${BRAND_CONFIG_FILES[@]}; do
      sudo ln -sf "${PWD}/opczone/brand/${brand}/${cfg_file}" "${BRAND_DIR}/${brand}/${cfg_file}"
    done
  done
fi
