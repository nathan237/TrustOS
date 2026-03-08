#!/usr/bin/env python3
"""
DFU Mode Entry Assistant
========================
Real-time USB monitoring + step-by-step guide for iPhone 11 Pro DFU entry.
Watches USB and tells you exactly what to do and when.
"""

import time
import sys
import libusb_package
import usb.core
import usb.backend.libusb1

APPLE_VID = 0x05AC
NORMAL_PIDS = [0x12A8, 0x12A9, 0x12AB]  # Various normal mode PIDs
RECOVERY_PID = 0x1281
DFU_PID = 0x1227

def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def find_apple_device(backend):
    """Find any Apple device and return (device, mode)"""
    devs = list(usb.core.find(idVendor=APPLE_VID, find_all=True, backend=backend))
    for d in devs:
        if d.idProduct == DFU_PID:
            return d, "DFU"
        elif d.idProduct == RECOVERY_PID:
            return d, "Recovery"
        elif d.idProduct in NORMAL_PIDS:
            return d, "Normal"
    # Check for any Apple device
    if devs:
        return devs[0], f"Unknown(0x{devs[0].idProduct:04x})"
    return None, None

def clear_line():
    sys.stdout.write('\r' + ' ' * 80 + '\r')
    sys.stdout.flush()

def print_step(msg, prefix=">>>"):
    clear_line()
    print(f"\n  {prefix} {msg}")
    sys.stdout.flush()

def print_status(msg):
    sys.stdout.write(f'\r  ... {msg}' + ' ' * 20)
    sys.stdout.flush()

def wait_for_device(backend, target_mode=None, timeout=60):
    """Wait for an Apple device, optionally a specific mode"""
    start = time.time()
    last_msg = ""
    while time.time() - start < timeout:
        dev, mode = find_apple_device(backend)
        if dev:
            if target_mode is None or mode == target_mode:
                return dev, mode
            msg = f"Found {mode} mode (waiting for {target_mode}...)"
            if msg != last_msg:
                print_status(msg)
                last_msg = msg
        else:
            msg = "No device detected..."
            if msg != last_msg:
                print_status(msg)
                last_msg = msg
        time.sleep(0.5)
    return None, None

def wait_for_disconnect(backend, timeout=30):
    """Wait for device to disconnect"""
    start = time.time()
    while time.time() - start < timeout:
        dev, mode = find_apple_device(backend)
        if not dev:
            return True
        time.sleep(0.3)
    return False

def main():
    print()
    print("=" * 60)
    print("  DFU MODE ENTRY ASSISTANT - iPhone 11 Pro (A13)")
    print("  Real-time USB monitoring")
    print("=" * 60)
    print()
    
    backend = get_backend()
    if not backend:
        print("  ERREUR: libusb backend not found!")
        return False
    
    # Step 0: Check current state
    print("  [SCAN] Recherche de l'iPhone...")
    dev, mode = find_apple_device(backend)
    
    if mode == "DFU":
        print(f"\n  DEJA EN DFU! (0x{dev.idProduct:04x})")
        print("  Pret pour checkm8!")
        return True
    
    if dev:
        print(f"  iPhone detecte en mode: {mode} (PID=0x{dev.idProduct:04x})")
    else:
        print("  Aucun iPhone detecte.")
        print_step("Branche ton iPhone avec le cable USB et attends...", "!!!")
        dev, mode = wait_for_device(backend, timeout=120)
        if not dev:
            print("\n  TIMEOUT: Aucun device detecte apres 2 minutes.")
            print("  Verifie le cable et le port USB.")
            return False
        print(f"\n  iPhone detecte! Mode: {mode}")
    
    # If in Recovery, we can skip to DFU entry from Recovery
    if mode == "Recovery":
        print()
        print("  L'iPhone est en Recovery. On entre en DFU depuis Recovery.")
        print()
        print("  PRET? Fais exactement ceci:")
        print()
        print("  1. Maintiens LATERAL + VOLUME BAS ensemble")
        input("     >>> Appuie sur ENTREE quand tu les tiens <<<")
        
        print()
        print("  2. Continue de tenir... 10 secondes")
        for i in range(10, 0, -1):
            print_status(f"Tiens les boutons... {i}s")
            time.sleep(1)
        
        print()
        print_step("3. RELACHE LE BOUTON LATERAL maintenant!", "!!!")
        print("     Mais GARDE Volume Bas enfonce!")
        input("     >>> Appuie ENTREE quand Lateral est relache <<<")
        
        print()
        print("  4. Continue de tenir Volume Bas... 10 secondes")
        for i in range(10, 0, -1):
            dev2, mode2 = find_apple_device(backend)
            if mode2 == "DFU":
                print()
                print(f"\n  *** DFU DETECTE! *** (0x{dev2.idProduct:04x})")
                print("  Tu peux relacher Volume Bas!")
                print("  SUCCES!")
                return True
            print_status(f"Volume Bas enfonce... {i}s")
            time.sleep(1)
        
        # Check one more time
        time.sleep(2)
        dev2, mode2 = find_apple_device(backend)
        if mode2 == "DFU":
            print(f"\n  *** DFU DETECTE! *** SUCCESS!")
            return True
        else:
            print(f"\n  Mode actuel: {mode2 or 'aucun device'}")
            print("  DFU non detecte. On reessaie...")
    
    # DFU entry from Normal/powered on state
    if mode in ("Normal", None) or (mode and mode.startswith("Unknown")):
        print()
        print("  === ENTREE EN DFU DEPUIS iOS ===")
        print()
        print("  C'est une sequence precise. Je te guide en temps reel.")
        print("  L'ecran va rester NOIR si ca marche (pas de logo Apple).")
        print()
        input("  >>> Appuie ENTREE quand tu es pret <<<")
        
        print()
        print("  ETAPE 1: Appuie VOLUME HAUT puis relache vite")
        time.sleep(1.5)
        
        print("  ETAPE 2: Appuie VOLUME BAS puis relache vite")
        time.sleep(1.5)
        
        print("  ETAPE 3: MAINTIENS le bouton LATERAL (ne relache pas!)")
        input("     >>> ENTREE quand tu tiens le Lateral <<<")
        
        print()
        print("  Continue de tenir Lateral... attente ecran noir")
        for i in range(10, 0, -1):
            print_status(f"Tiens Lateral... {i}s")
            time.sleep(1)
        
        print()
        print_step("MAINTENANT: Ajoute VOLUME BAS (tiens les 2!)", "!!!")
        input("     >>> ENTREE quand tu tiens Lateral + Vol Bas <<<")
        
        print()
        print("  Tiens les deux boutons 5 secondes...")
        for i in range(5, 0, -1):
            print_status(f"Lateral + Vol Bas... {i}s")
            time.sleep(1)
        
        print()
        print_step("RELACHE LATERAL mais GARDE Volume Bas!", "!!!")
        input("     >>> ENTREE quand Lateral est relache <<<")
        
        print()
        print("  Tiens seulement Volume Bas... surveillance USB...")
        for i in range(15, 0, -1):
            dev2, mode2 = find_apple_device(backend)
            if mode2 == "DFU":
                print()
                print(f"\n  *** DFU DETECTE! PID=0x{dev2.idProduct:04x} ***")
                print("  Tu peux relacher Volume Bas!")
                print()
                print("  ==============================")
                print("  ===   SUCCES - MODE DFU!   ===")
                print("  ==============================")
                return True
            if mode2 == "Recovery":
                print()
                print("  Recovery detecte au lieu de DFU.")
                print("  Le timing n'etait pas bon. On reessaie.")
                break
            print_status(f"Volume Bas enfonce... {i}s (USB scan...)")
            time.sleep(1)
        
        # Final check
        time.sleep(3)
        dev2, mode2 = find_apple_device(backend)
        if mode2 == "DFU":
            print(f"\n  *** SUCCES! DFU MODE! ***")
            return True
        elif mode2 == "Recovery":
            print("\n  En Recovery - le Lateral a ete tenu trop longtemps.")
            print("  Reessaie: relache Lateral EXACTEMENT apres 5 sec.")
        elif mode2:
            print(f"\n  Mode: {mode2}")
        else:
            print("\n  Aucun device. L'iPhone a peut-etre reboot en iOS.")
            print("  Rebranche et relance le script.")
    
    print()
    print("  DFU non atteint. Relance: python dfu_assistant.py")
    return False

if __name__ == "__main__":
    success = main()
    if success:
        print()
        print("  Prochaine etape: python live_test.py --probe")
    print()
