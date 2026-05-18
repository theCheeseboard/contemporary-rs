#!/bin/bash

crates=(
  "localesupport/cntp_localesupport"
  "i18n/cntp_i18n_core"
  "i18n/cntp_i18n_build_core"
  "i18n/cntp_i18n_parse"
  "i18n/cntp_i18n_gen"
  "i18n/cntp_i18n_macros"
  "i18n/cntp_i18n"
  "i18n/cargo_cntp_i18n"
  "i18n/cntp_i18n_parlance_source"

  "cntp_config"
  "icon_tool/cntp_icon_tool_core"
  "icon_tool/cntp_icon_tool_macros"
  "deploy_tool/cntp_bundle_lib"
  "deploy_tool/cargo_cntp_deploy"
  "deploy_tool/cargo_cntp_bundle"
)

function publish_crate() {
    echo "Publishing crate $1"
    pushd "$1" > /dev/null || return

    if ! cargo_output=$(cargo publish --no-verify 2>&1); then
      echo "$cargo_output"
      echo "Crate $1 could not be published"
      exit 1
    fi

    popd > /dev/null || exit
}

for crate in "${crates[@]}"; do
  publish_crate "$crate"
done