#!/bin/sh
# Instala o actualiza Angri (la simulación, en Rust). Se corre como fran desde ~/despliegue.
# Espera: ~/despliegue/Angri/ (el repo entero, con rust/ y deploy/ adentro).
# Necesita cargo: sudo apt install cargo (Debian 13 trae Rust 1.85, alcanza).
# La mente (/var/lib/angri/mente.txt) no se toca: al arrancar, Angri retoma donde estaba.
# La primera instalación la deja apagada; se prende a mano (ver abajo).
# La primera vez después de la versión en Python, lo de Python (mente.json, vida.csv,
# respaldos, foto.json) se mueve a /var/lib/angri/python-viejo/ y Angri renace en Rust
# (Rust no lee las mentes de Python).
set -e
D="$HOME/despliegue/Angri"

command -v cargo >/dev/null || { echo "Falta cargo: sudo apt install cargo"; exit 1; }
echo "Compilando (un par de minutos la primera vez)..."
(cd "$D/rust" && cargo build --release --locked)
(cd "$D/rust" && cargo test --release --locked) || { echo "Las pruebas fallaron: no instalo."; exit 1; }

id angri >/dev/null 2>&1 || sudo useradd --system --home-dir /var/lib/angri --shell /usr/sbin/nologin angri

# Si ya vivía, que guarde antes de cambiarle el código
if systemctl is-active -q angri.service 2>/dev/null; then sudo systemctl stop angri.service; fi
if systemctl is-active -q angri-visor.service 2>/dev/null; then sudo systemctl stop angri-visor.service; fi

sudo install -d -m 755 /opt/angri
sudo rm -rf /opt/angri/codigo          # (la versión en Python)
sudo install -m 755 "$D/rust/target/release/angri" /opt/angri/angri

sudo install -d -o angri -g angri -m 750 /var/lib/angri
# (con sudo test: fran no puede mirar adentro de /var/lib/angri)
if sudo test -e /var/lib/angri/mente.json && ! sudo test -e /var/lib/angri/mente.txt; then
  V=/var/lib/angri/python-viejo
  sudo install -d -o angri -g angri -m 750 "$V"
  for f in mente.json mente.json.tmp vida.csv foto.json respaldos; do
    if sudo test -e "/var/lib/angri/$f"; then sudo mv "/var/lib/angri/$f" "$V/"; fi
  done
  echo "Lo de Python quedó en $V; Angri renace en Rust."
fi

sudo install -m 644 "$D/deploy/angri.service" /etc/systemd/system/
sudo install -m 644 "$D/deploy/angri-visor.service" /etc/systemd/system/
sudo install -m 755 "$D/deploy/angri" /usr/local/bin/angri
sudo systemctl daemon-reload
# La vida y el visor solo se prenden si ya estaban habilitados: si Angri está
# apagada a propósito (sudo systemctl disable --now angri angri-visor), actualizar
# no la despierta. Para prenderla: sudo systemctl enable --now angri angri-visor
# visor web: http://angri.local:8080 (avahi anuncia angri.local si el host se llama "angri").
for u in angri.service angri-visor.service; do
  if systemctl is-enabled -q "$u" 2>/dev/null; then sudo systemctl start "$u"; else echo "$u queda apagado"; fi
done
if systemctl is-active -q angri.service; then
  sleep 3
  sudo journalctl -u angri -n 3 --no-pager -o cat
fi
systemctl is-active angri-visor.service >/dev/null && echo "Visor: http://$(hostname).local:8080"
echo INSTALADO
