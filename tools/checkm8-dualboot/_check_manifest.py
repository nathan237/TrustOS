import plistlib
with open('tools/checkm8-dualboot/cache/3a4b009f41c729764f254068727c5545_BuildManifest.plist', 'rb') as f:
    m = plistlib.load(f)
identity = m['BuildIdentities'][0]
info = identity.get('Info', {})
print('Identity Info:')
for k, v in sorted(info.items()):
    if isinstance(v, bytes): v = v.hex()[:40]
    print(f'  {k}: {v}')
print()
print(f"ApBoardID: {identity.get('ApBoardID', None)}")
print(f"ApChipID: {identity.get('ApChipID', None)}")
print(f"ProductVersion: {m.get('ProductVersion', '?')}")  
print(f"ProductBuildVersion: {m.get('ProductBuildVersion', '?')}")
print(f"SupportedProductTypes: {m.get('SupportedProductTypes', [])}")
