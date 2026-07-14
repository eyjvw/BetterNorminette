# BetterNorminette

**[English](#english)** · **[Français](#français)**

---

## English

A pretty, multilingual front-end for the 42 `norminette`. Same checks, same
codes — but readable output, grouped by file, with every one of the 138
norminette messages translated into **English, French and Spanish**.

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/eyjvw/BetterNorminette/main/install.sh | sh
```

Installs the prebuilt binary (Linux x86_64/arm64, macOS Intel/Apple Silicon)
into `~/.betternorminette` and creates the `better-norminette` command plus
the short `bnorm` alias in `~/.local/bin`. Requires `norminette` itself
(`pipx install norminette`).

### Usage

```sh
bnorm                 # check the current directory
bnorm src/ ft_split.c # check specific paths
bnorm -l es           # Spanish for this run
bnorm lang fr         # set the default language (persisted)
```

Language resolution order: `-l/--lang` flag → `BETTERNORMINETTE_LANG` env →
saved config (`bnorm lang <x>`) → system `$LANG` → English.

Exit code: 0 when the norm is clean, 1 when there are errors — usable in
scripts and git hooks.

### Auto-update / uninstall

Same mechanics as MiniMoulinette: daily check with a `[Y/n]` prompt,
`bnorm update` to force, `BETTERNORMINETTE_NO_UPDATE=1` to disable,
`bnorm uninstall` to remove everything.

---

## Français

Un front-end joli et multilingue pour la `norminette` de 42. Mêmes checks,
mêmes codes — mais une sortie lisible, groupée par fichier, avec les 138
messages de la norminette traduits en **anglais, français et espagnol**.

### Installation

```sh
curl -fsSL https://raw.githubusercontent.com/eyjvw/BetterNorminette/main/install.sh | sh
```

Installe le binaire précompilé (Linux x86_64/arm64, macOS Intel/Apple
Silicon) dans `~/.betternorminette` et crée la commande `better-norminette`
plus l'alias court `bnorm` dans `~/.local/bin`. Nécessite `norminette`
(`pipx install norminette`).

### Utilisation

```sh
bnorm                 # vérifie le dossier courant
bnorm src/ ft_split.c # vérifie des chemins précis
bnorm -l es           # espagnol pour ce run
bnorm lang fr         # définit la langue par défaut (persistée)
```

Ordre de résolution de la langue : flag `-l/--lang` → env
`BETTERNORMINETTE_LANG` → config sauvegardée (`bnorm lang <x>`) → `$LANG`
système → anglais.

Code de sortie : 0 si la norme est propre, 1 s'il y a des erreurs —
utilisable en script et hook git.

### Mise à jour / désinstallation

Même mécanique que MiniMoulinette : check quotidien avec prompt `[Y/n]`,
`bnorm update` pour forcer, `BETTERNORMINETTE_NO_UPDATE=1` pour désactiver,
`bnorm uninstall` pour tout supprimer.
