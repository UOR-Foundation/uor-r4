#!/usr/bin/env bash
set -euo pipefail

list_files() {
    printf '%s\n' \
        'index.html' \
        'index.css' \
        'geometric_prime_router_webapp.html' \
        'coi-serviceworker.js' \
        'r4_worker.js' \
        'models.json' \
        'assets/css/router_demo_offline.css' \
        'assets/css/uor_offline_fonts.css' \
        'assets/js/composer_input.js' \
        'assets/js/desktop_actions.js' \
        'assets/js/desktop_openai.js' \
        'assets/js/desktop_origin.js' \
        'assets/js/image_input.js' \
        'assets/js/mcp_client.js' \
        'assets/js/model_sources.js' \
        'assets/js/stream_metrics.js' \
        'assets/js/studio_api.js' \
        'assets/js/studio_files.js' \
        'assets/js/studio_ui.js' \
        'assets/js/uor_model_worker.js' \
        'assets/js/uor_offline_vendor.js' \
        'assets/js/uor_tokenizer_artifact.js' \
        'pkg/uor_r4_integer.js' \
        'pkg/uor_r4_integer_bg.wasm' \
        'pkg/synthetic-stack.lut' \
        'pkg/tokenizer.json'
}

case "${1:-}" in
    --list)
        list_files
        ;;
    copy)
        [ "$#" -ge 3 ] || { printf 'usage: runtime-files.sh copy <source> <dest> [--allow-generated-missing]\n' >&2; exit 2; }
        src=$2
        dest=$3
        allow_missing=${4:-}
        [ -d "$src" ] || { printf 'Studio source is unavailable: %s\n' "$src" >&2; exit 1; }
        case "$dest" in ''|'/'|'.') printf 'Refusing unsafe destination: %s\n' "$dest" >&2; exit 1 ;; esac
        rm -rf "$dest"
        while IFS= read -r rel; do
            if [ -f "$src/$rel" ]; then
                mkdir -p "$(dirname "$dest/$rel")"
                cp "$src/$rel" "$dest/$rel"
            elif [ "$allow_missing" != '--allow-generated-missing' ]; then
                printf 'Required runtime file is missing: %s\n' "$src/$rel" >&2
                exit 1
            fi
        done < <(list_files)
        ;;
    --verify)
        [ "$#" -eq 2 ] || { printf 'usage: runtime-files.sh --verify <dest>\n' >&2; exit 2; }
        dest=$2
        while IFS= read -r rel; do
            [ -f "$dest/$rel" ] || { printf 'Runtime file is missing: %s\n' "$dest/$rel" >&2; exit 1; }
        done < <(list_files)
        printf 'Runtime allowlist verified: %s\n' "$dest"
        ;;
    *)
        printf 'usage: runtime-files.sh --list | copy <source> <dest> [--allow-generated-missing] | --verify <dest>\n' >&2
        exit 2
        ;;
esac
