#!/bin/sh
# BetterNorminette uninstaller.
#
#   curl -fsSL https://raw.githubusercontent.com/eyjvw/BetterNorminette/main/uninstall.sh | sh
set -e

INSTALL_DIR="${BETTERNORMINETTE_DIR:-$HOME/.betternorminette}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

info() { printf '\033[1;36m==>\033[0m %s\n' "$1"; }

removed=0
if [ -d "$INSTALL_DIR" ]; then
	rm -rf "$INSTALL_DIR"
	info "Removed $INSTALL_DIR"
	removed=1
fi
for cmd in better-norminette bnorm; do
	if [ -e "$BIN_DIR/$cmd" ] || [ -L "$BIN_DIR/$cmd" ]; then
		rm -f "$BIN_DIR/$cmd"
		info "Removed $BIN_DIR/$cmd"
		removed=1
	fi
done

marker='# added by better-norminette installer'
export_line="export PATH=\"$BIN_DIR:\$PATH\""
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
	[ -f "$rc" ] || continue
	if grep -qxF "$marker" "$rc"; then
		grep -vxF "$marker" "$rc" | grep -vxF "$export_line" > "$rc.bn_tmp"
		mv "$rc.bn_tmp" "$rc"
		info "Cleaned PATH lines from $rc"
		removed=1
	fi
done

if [ "$removed" -eq 1 ]; then
	info "better-norminette uninstalled. Bye!"
else
	info "nothing to uninstall (better-norminette was not installed)"
fi
