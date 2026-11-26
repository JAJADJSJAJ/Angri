#!/usr/bin/env bash
# Lanza `angri comparar` en GitHub Actions (.github/workflows/medir.yml), espera y baja la tabla.
#   rust/medir-actions.sh TICKS PARES "AJUSTES A" "AJUSTES B" [PARTES] [DESDE]
#   rust/medir-actions.sh 300000 256 "" "MUNDO=bosque" 16 0
# Necesita `gh` con sesión iniciada. La tabla queda en medicion-<id>/tabla.txt y los
# crudos de cada porción en medicion-<id>/crudos/ (se rejuntan con `angri juntar`).
#
# A mano, lo mismo es:
#   gh workflow run medir.yml -f ticks=300000 -f pares=256 -f variante_a="" -f variante_b="MUNDO=bosque" -f partes=16
#   gh run list --workflow medir.yml -L 1          # el id de la corrida
#   gh run watch <id>
#   gh run download <id> -n tabla -D medicion-<id>
set -euo pipefail
[ $# -ge 4 ] || { sed -n 2,4p "$0"; exit 1; }
ticks=$1; pares=$2; a=$3; b=$4; partes=${5:-16}; desde=${6:-0}
antes=$(gh run list --workflow medir.yml -L 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
gh workflow run medir.yml -f ticks="$ticks" -f pares="$pares" -f desde="$desde" \
  -f variante_a="$a" -f variante_b="$b" -f partes="$partes"
id=""
for _ in $(seq 30); do
  sleep 2
  id=$(gh run list --workflow medir.yml -L 1 --json databaseId -q '.[0].databaseId')
  [ -n "$id" ] && [ "$id" != "$antes" ] && break
done
echo "corrida $id"
gh run watch "$id" --exit-status || true
gh run download "$id" -n tabla -D "medicion-$id"
cat "medicion-$id/tabla.txt"
