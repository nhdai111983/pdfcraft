# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                    workspace root
#   VERSION                 [workspace.package] version from Cargo.toml (override: PDFCRAFT_VERSION)
#   PDFCRAFT_BUILD_LABEL    optional label for a local or patched build (letters, digits, dots);
#                           compiled into `--version` and About, and appended to the package release
#   DIST                    output directory for release artifacts (default: $ROOT/dist/release)
#   PDFCRAFT_BUILD_SHA    git commit, recorded in the macOS Info.plist (PdfCraftBuildCommit)
#   PDFCRAFT_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR        cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
}

VERSION="${PDFCRAFT_VERSION:-$(workspace_version)}"
if [ -z "$VERSION" ]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

# A labelled build says so in `--version` and About (pdfcraft_engine::build_info) and in its
# package release (packaging/linux/nfpm.yaml), so it can't be mistaken for the release.
if [ -n "${PDFCRAFT_BUILD_LABEL:-}" ]; then
  case "$PDFCRAFT_BUILD_LABEL" in
    *[!A-Za-z0-9.]* | .* | *.)
      echo "error: PDFCRAFT_BUILD_LABEL may hold only letters, digits and inner dots: $PDFCRAFT_BUILD_LABEL" >&2
      exit 1
      ;;
  esac
  export PDFCRAFT_BUILD_LABEL
fi

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [ -z "${PDFCRAFT_BUILD_SHA:-}" ]; then
  PDFCRAFT_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export PDFCRAFT_BUILD_SHA
export PDFCRAFT_BUILD_DATE="${PDFCRAFT_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
}

# Copy licence and readme files that exist into a package directory.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE COPYRIGHT; do
    if [ -f "$ROOT/$f" ]; then cp "$ROOT/$f" "$dest/" || return 1; fi
  done
  copy_font_licences "$dest" || return 1
  copy_notices "$dest" || return 1
}

# NexusOffice: the licences, NOTICE (Apache-2.0 §4(d)) and ATTRIBUTION.md (the bundled fonts and
# assets with their licences) go in every package; the branded README links to NOTICE and
# ATTRIBUTION.md. So do the OFL texts of the repo's own embedded fonts (assets/fonts/OFL-*.txt).
# Run after copy_font_licences: a name that clashes with a craft-fonts OFL-<family>.txt on a
# case-insensitive filesystem then fails instead of overwriting it. Every copy is checked, because
# callers use `copy_notices … || …`, which turns errexit off inside the function.
copy_notices() {
  local dest="$1" f
  for f in LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md; do
    if [ ! -f "$ROOT/$f" ]; then echo "error: $ROOT/$f is missing" >&2; return 1; fi
    cp "$ROOT/$f" "$dest/" || return 1
  done
  for f in "$ROOT"/assets/fonts/OFL-*.txt; do
    [ -f "$f" ] || continue
    if [ -e "$dest/${f##*/}" ]; then echo "error: $dest/${f##*/} already exists (font licence name clash)" >&2; return 1; fi
    cp "$f" "$dest/" || return 1
  done
  # pdfcraft's other third-party notices, under the names the MSI's licenses\ folder uses: the
  # Lucide icons (ISC, plus MIT for the Feather-derived ones), the Foxit standard-14 fonts
  # embedded by vendored hayro-interpret (BSD-3-Clause) and the Adobe CMap resources hayro-cmap
  # compiles in (BSD-3-Clause). Required.
  local pair src name
  for pair in assets/icons/LICENSE-lucide.txt:LICENSE-lucide.txt \
    vendor/hayro-interpret/assets/LICENSE_FOXIT:LICENSE-FOXIT.txt \
    assets/licenses/LICENSE-cmap-resources.txt:LICENSE-cmap-resources.txt; do
    src="${pair%%:*}" name="${pair##*:}"
    if [ ! -f "$ROOT/$src" ]; then echo "error: $ROOT/$src is missing" >&2; return 1; fi
    if [ -e "$dest/$name" ]; then echo "error: $dest/$name already exists (licence name clash)" >&2; return 1; fi
    cp "$ROOT/$src" "$dest/$name" || return 1
  done
}

# Builds made with the optional craft-fonts input (CRAFT_FONTS_DIR, set for every release) embed
# its fonts, so the package carries their licences: fonts/<family>/OFL.txt -> OFL-<family>.txt.
# NexusOffice: fails when CRAFT_FONTS_REQUIRED=1 (every release) finds none. The repo's own
# fonts (assets/fonts/OFL-*.txt) and the other third-party notices come from copy_notices, which
# copy_docs and the macOS .app call after this.
copy_font_licences() {
  local dest="$1" f family n=0
  if [ -n "${CRAFT_FONTS_DIR:-}" ]; then
    for f in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
      [ -f "$f" ] || continue
      family="$(basename "$(dirname "$f")")"
      cp "$f" "$dest/OFL-$family.txt" || return 1
      n=$((n + 1))
    done
  fi
  # NexusOffice: a release build (CRAFT_FONTS_REQUIRED=1) embeds craft-fonts, so its licences must ship.
  if [ "${CRAFT_FONTS_REQUIRED:-}" = 1 ] && [ "$n" -eq 0 ]; then
    echo "error: CRAFT_FONTS_REQUIRED=1 but no fonts/*/OFL.txt in CRAFT_FONTS_DIR (${CRAFT_FONTS_DIR:-unset})" >&2
    return 1
  fi
}

# Fetch the OCR models into a package directory (#103): `cargo xtask models DEST` downloads every
# ATTRIBUTION.toml `kind = "model"` file, verified by SHA-256, with its licence text and an
# ATTRIBUTION.txt crediting them. DEST must be where pdfcraft_ocr::Models::dirs_beside_exe looks
# from the installed executable. Nothing here names a model file, so a model added to the manifest
# later ships without changing the packaging.
stage_models() {
  local dest="$1"
  mkdir -p "$dest"
  dest="$(cd "$dest" && pwd)"
  (cd "$ROOT" && cargo xtask models "$dest")
  rm -f "$dest"/*.part
  if ! ls "$dest"/*.LICENCE.txt >/dev/null 2>&1 || [ ! -s "$dest/ATTRIBUTION.txt" ]; then
    echo "error: no OCR models in $dest after cargo xtask models" >&2
    exit 1
  fi
  echo "OCR models in $dest:"
  ls -l "$dest"
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}
