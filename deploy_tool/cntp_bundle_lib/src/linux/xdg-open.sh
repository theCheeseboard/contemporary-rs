#!/bin/bash

# Find the system xdg-open
mapfile -t xdg_open_paths < <(which -a xdg-open)

if [[ ${#xdg_open_paths} -ge 2 ]]; then
  xdg_open_path="${xdg_open_paths[1]}"

  export XDG_DATA_DIRS="$CNTP_RUNTIME_OLD_XDG_DATA_DIRS"
  export LD_LIBRARY_PATH="$CNTP_RUNTIME_OLD_LD_LIBRARY_PATH"
  export PATH="$CNTP_RUNTIME_OLD_PATH"

  exec "$xdg_open_path" "${@}"
else
  echo "xdg-open: Unable to find system xdg-open" >&2
  exit 1
fi

