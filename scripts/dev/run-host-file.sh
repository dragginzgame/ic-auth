#!/usr/bin/env bash
set -euo pipefail
script_path="${BASH_SOURCE[0]}"
[[ "$script_path" == /* ]] || script_path="$PWD/$script_path"
root="$(cd -P "${script_path%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# Admission is offline; preserve the invocation directory and OS-native arguments.
tool="$(bash "$root/scripts/dev/host-file-tools.sh" check)"
exec "$tool" "$@"
