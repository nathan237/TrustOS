#!/bin/bash
chmod 666 /dev/kvm
rm -f /tmp/kvm_serial.log
# Navigate to workspace root (portable)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
(
  sleep 25
  echo 'sendkey h'
  sleep 0.1
  echo 'sendkey v'
  sleep 0.1
  echo 'sendkey space'
  sleep 0.1
  echo 'sendkey i'
  sleep 0.1
  echo 'sendkey n'
  sleep 0.1
  echo 'sendkey i'
  sleep 0.1
  echo 'sendkey t'
  sleep 0.1
  echo 'sendkey ret'
  sleep 10
  echo 'quit'
) | timeout 45 qemu-system-x86_64 -cdrom trustos.iso -m 512M -enable-kvm -cpu host \
  -chardev file,id=serial0,path=/tmp/kvm_serial.log -serial chardev:serial0 \
  -display none -monitor stdio -no-reboot
cat /tmp/kvm_serial.log
