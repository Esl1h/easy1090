#!/usr/bin/env bash
#===============================================================================
# easy1090 - install wrapper
#
# Kept so that "git clone && ./install.sh" keeps working. The real entrypoint
# is ./easy1090, which also does status, start/stop/restart, open and uninstall.
#
# Author: Esli
# License: MIT
#===============================================================================

set -euo pipefail

exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/easy1090" install "$@"
