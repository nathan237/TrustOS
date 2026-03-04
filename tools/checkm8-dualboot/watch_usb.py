import time, subprocess

print("Monitoring PnP for 60 seconds - REPLUG THE CABLE NOW")
print("=" * 50)
seen = set()
for i in range(120):
    r = subprocess.run(
        ["powershell", "-c", 'Get-PnpDevice -PresentOnly | Where-Object { $_.InstanceId -match "05AC" } | Select-Object -ExpandProperty InstanceId'],
        capture_output=True, text=True, timeout=5
    )
    for line in r.stdout.strip().split("\n"):
        line = line.strip()
        if line and line not in seen:
            seen.add(line)
            print(f"  [{i*0.5:.1f}s] FOUND: {line}")
    time.sleep(0.5)
print(f"Done. Found {len(seen)} Apple devices.")
