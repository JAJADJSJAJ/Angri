"""Arranca la vida de Angri.

    python vivir.py              -> vive para siempre, con panel
    python vivir.py --rapido N   -> vive N ticks sin panel (para probar)
"""
import sys

from angri.vida import vivir

if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--rapido":
        a = vivir(ticks=int(sys.argv[2]), mostrar=False)
        print(f"Vivió {a.edad} ticks, durmió {a.suenos} veces. Malestar: {a.cuerpo.malestar():.2f}")
    else:
        vivir()
