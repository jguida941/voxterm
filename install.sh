#!/bin/bash
#
# VoxTerm Installer (overlay mode)
# Run: ./install.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/scripts/setup.sh" install "$@"
