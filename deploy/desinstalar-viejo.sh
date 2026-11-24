#!/bin/sh
# Borra el proyecto viejo del i5 (Ai Project: llama.cpp, Qwen, ciclo/sueño/web)
# para que Angri (la simulación) ocupe su lugar. Se corre UNA vez, como fran,
# desde ~/despliegue, ANTES de instalar.sh. No guarda nada del proyecto viejo.
# NO toca: el usuario angri (se reutiliza), las actualizaciones de seguridad
# (unattended-upgrades, reinicio 05:30, apt-daily-upgrade.timer.d/angri.conf)
# ni la regla udev del wifi (70-mercusys-ma30n.rules).
set -e

echo "Parando y borrando servicios viejos:"
for u in angri-ciclo.timer angri-sueno.timer angri-ciclo.service angri-sueno.service \
         angri-web.service llama-charla.service llama-emb.service llama-dia.service; do
  if [ -e "/etc/systemd/system/$u" ] || systemctl list-unit-files -q "$u" 2>/dev/null | grep -q .; then
    sudo systemctl disable --now "$u" 2>/dev/null || true
    sudo rm -f "/etc/systemd/system/$u"
    sudo rm -rf "/etc/systemd/system/$u.d"
    echo "  $u"
  fi
done
sudo systemctl daemon-reload
sudo systemctl reset-failed 2>/dev/null || true

borrar() {
  if [ -e "$1" ]; then sudo rm -rf "$1"; echo "  $1"; fi
}
echo "Borrando archivos viejos:"
borrar /opt/angri
borrar /etc/ld.so.conf.d/angri-llama.conf
sudo ldconfig
borrar /usr/local/bin/angri
borrar "$HOME/llama.cpp"
borrar "$HOME/modelos"
if [ -d /var/lib/angri ]; then
  sudo find /var/lib/angri -mindepth 1 -delete
  echo "  contenido de /var/lib/angri"
fi
echo VIEJO_BORRADO
