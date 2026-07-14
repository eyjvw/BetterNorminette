#!/bin/sh
# BetterNorminette installer.
#
#   curl -fsSL https://raw.githubusercontent.com/eyjvw/BetterNorminette/main/install.sh | sh
#
# Installs the prebuilt binary into ~/.betternorminette and links the
# better-norminette (+ bnorm alias) commands into ~/.local/bin.
# Requires norminette itself (pipx install norminette).
set -e

REPO="eyjvw/BetterNorminette"
INSTALL_DIR="${BETTERNORMINETTE_DIR:-$HOME/.betternorminette}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

info() { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
err() { printf '\033[1;31merror:\033[0m %s\n' "$1" >&2; }

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
	Linux)
		case "$arch" in
			x86_64) target="x86_64-unknown-linux-musl" ;;
			aarch64 | arm64) target="aarch64-unknown-linux-musl" ;;
			*) target="" ;;
		esac
		;;
	Darwin)
		case "$arch" in
			x86_64) target="x86_64-apple-darwin" ;;
			arm64) target="aarch64-apple-darwin" ;;
			*) target="" ;;
		esac
		;;
	*) target="" ;;
esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

info "Platform: $os/$arch -> ${target:-unsupported}"

latest="$(curl -fsSLI --retry 3 --retry-all-errors -o /dev/null -w '%{url_effective}' \
	"https://github.com/$REPO/releases/latest" 2>/dev/null | sed 's|.*/||')"
if [ -n "$latest" ] && [ "$latest" != "latest" ]; then
	info "Latest release: $latest"
else
	err "could not resolve the latest release (network/proxy issue?)"
	latest=""
fi

fetched=0
if [ -n "$target" ]; then
	if [ -n "$latest" ]; then
		url="https://github.com/$REPO/releases/download/$latest/better-norminette-$target.tar.gz"
	else
		url="https://github.com/$REPO/releases/latest/download/better-norminette-$target.tar.gz"
	fi
	info "Downloading better-norminette ($target)..."
	if curl -fSL --retry 3 --retry-all-errors --progress-bar "$url" -o "$tmp/bn.tar.gz"; then
		fetched=1
	else
		err "download failed: $url"
	fi
fi

if [ "$fetched" -eq 1 ]; then
	mkdir -p "$tmp/pkg"
	tar -xzf "$tmp/bn.tar.gz" -C "$tmp/pkg"
else
	if ! command -v cargo >/dev/null 2>&1; then
		err "no prebuilt binary for $os/$arch and cargo is not installed."
		err "install rust (https://rustup.rs) and re-run, or build manually."
		exit 1
	fi
	info "Building from source with cargo (this can take a minute)..."
	git clone --depth 1 "https://github.com/$REPO" "$tmp/src" >/dev/null 2>&1
	(cd "$tmp/src" && cargo build --release --quiet)
	mkdir -p "$tmp/pkg"
	cp "$tmp/src/target/release/better-norminette" "$tmp/pkg/better-norminette"
fi

info "Installing into $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
rm -f "$INSTALL_DIR/better-norminette"
cp "$tmp/pkg/better-norminette" "$INSTALL_DIR/better-norminette"
chmod +x "$INSTALL_DIR/better-norminette"

mkdir -p "$BIN_DIR"
ln -sf "$INSTALL_DIR/better-norminette" "$BIN_DIR/better-norminette"
ln -sf "$INSTALL_DIR/better-norminette" "$BIN_DIR/bnorm"

installed_v="$("$INSTALL_DIR/better-norminette" version 2>/dev/null || echo '?')"
info "Installed: $BIN_DIR/better-norminette + bnorm alias ($installed_v)"

if ! command -v norminette >/dev/null 2>&1; then
	printf '\033[1;33mwarning:\033[0m norminette itself is not installed — better-norminette needs it.\n'
	printf 'install it with: pipx install norminette\n'
fi

# --- interactive setup (works through `curl | sh` thanks to /dev/tty) -------
ask()
{
	printf '%s' "$1" > /dev/tty
	read -r REPLY < /dev/tty
}

# default from the system locale
case "${LANG:-}" in
	fr*) sys_lang="fr" ;;
	es*) sys_lang="es" ;;
	*) sys_lang="en" ;;
esac

alias_marker='# added by better-norminette installer'
# first install only — updates must not re-ask
if [ ! -f "$INSTALL_DIR/lang" ] && [ -e /dev/tty ] && [ -r /dev/tty ] && [ -w /dev/tty ]; then
	# 1) default language
	ask "$(printf '\033[1;36m?\033[0m Default language / Langue par défaut / Idioma [en/fr/es] (%s): ' "$sys_lang")"
	lang="$(printf '%s' "$REPLY" | tr 'A-Z' 'a-z')"
	case "$lang" in
		en | fr | es) ;;
		*) lang="$sys_lang" ;;
	esac
	mkdir -p "$INSTALL_DIR"
	printf '%s' "$lang" > "$INSTALL_DIR/lang"
	info "Default language: $lang"

	# 2) alias norminette -> better-norminette
	ask "$(printf '\033[1;36m?\033[0m Alias \033[1mnorminette\033[0m to better-norminette in your shell? [y/N] ')"
	case "$(printf '%s' "$REPLY" | tr 'A-Z' 'a-z')" in
		y | yes | o | oui | s | si | sí)
			for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
				[ -f "$rc" ] || continue
				if ! grep -qxF 'alias norminette="better-norminette"' "$rc"; then
					printf '\n%s\nalias norminette="better-norminette"\n' "$alias_marker" >> "$rc"
					info "Alias added to $rc"
				fi
			done
			;;
		*) info "No alias — use better-norminette / bnorm" ;;
	esac
elif [ ! -f "$INSTALL_DIR/lang" ]; then
	# non-interactive first install: pick the system language, no alias
	mkdir -p "$INSTALL_DIR"
	printf '%s' "$sys_lang" > "$INSTALL_DIR/lang"
	info "Default language: $sys_lang (change it with: bnorm lang <en|fr|es>)"
fi

# add BIN_DIR to PATH in the shell rc files if missing
add_path_to_rc()
{
	rc="$1"
	line="export PATH=\"$BIN_DIR:\$PATH\""
	[ -f "$rc" ] || return 0
	if ! grep -Fq "$line" "$rc"; then
		printf '\n# added by better-norminette installer\n%s\n' "$line" >> "$rc"
		info "Added $BIN_DIR to PATH in $rc"
		UPDATED_RC=1
	fi
}

case ":$PATH:" in
	*":$BIN_DIR:"*) ;;
	*)
		UPDATED_RC=0
		add_path_to_rc "$HOME/.zshrc"
		add_path_to_rc "$HOME/.bashrc"
		if [ "$UPDATED_RC" -eq 0 ]; then
			case "${SHELL:-}" in
				*zsh) touch "$HOME/.zshrc" && add_path_to_rc "$HOME/.zshrc" ;;
				*) touch "$HOME/.bashrc" && add_path_to_rc "$HOME/.bashrc" ;;
			esac
		fi
		printf '\033[1;33mnote:\033[0m open a new terminal (or run: export PATH="%s:$PATH")\n' "$BIN_DIR"
		;;
esac

printf '\nUsage:\n'
printf '    bnorm                 # check the current directory\n'
printf '    bnorm -l fr src/      # in French\n'
printf '    bnorm lang es         # set the default language\n'
