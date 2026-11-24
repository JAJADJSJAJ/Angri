#!/bin/sh
# Instala o actualiza Angri (la simulación). Se corre como fran desde ~/despliegue.
# Espera: ~/despliegue/Angri/ (el repo entero, con deploy/ adentro).
# La mente (/var/lib/angri) no se toca: al arrancar, Angri retoma donde estaba.
set -e
D="$HOME/despliegue/Angri"

id angri >/dev/null 2>&1 || sudo useradd --system --home-dir /var/lib/angri --shell /usr/sbin/nologin angri

# Si ya vivía, que guarde antes de cambiarle el código
if systemctl is-active -q angri.service 2>/dev/null; then sudo systemctl stop angri.service; fi
if systemctl is-active -q angri-visor.service 2>/dev/null; then sudo systemctl stop angri-visor.service; fi

sudo rm -rf /opt/angri/codigo
sudo mkdir -p /opt/angri/codigo
sudo cp -r "$D/angri" "$D"/*.py /opt/angri/codigo/
sudo find /opt/angri/codigo -name '__pycache__' -prune -exec rm -rf {} +
sudo chown -R root:root /opt/angri
sudo chmod -R a+rX /opt/angri

sudo install -d -o angri -g angri -m 750 /var/lib/angri
sudo install -m 644 "$D/deploy/angri.service" /etc/systemd/system/
sudo install -m 644 "$D/deploy/angri-visor.service" /etc/systemd/system/
sudo install -m 755 "$D/deploy/angri" /usr/local/bin/angri
sudo systemctl daemon-reload
sudo systemctl enable --now angri.service
# visor web (etapa 13a): http://angri.local:8080 desde la red de casa.
# angri.local lo anuncia avahi (mDNS) si el host se llama "angri"; si no, usar la IP.
sudo systemctl enable --now angri-visor.service
sleep 3
sudo journalctl -u angri -n 3 --no-pager -o cat
systemctl is-active angri-visor.service >/dev/null && echo "Visor: http://$(hostname).local:8080"
echo INSTALADO
