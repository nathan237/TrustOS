#!/usr/bin/env python3
"""TSS request with real device nonce from DFU mode."""
import plistlib, asyncio, warnings
import urllib3
urllib3.disable_warnings()
from pymobiledevice3.restore.tss import TSSRequest

with open('tools/checkm8-dualboot/cache/3a4b009f41c729764f254068727c5545_BuildManifest.plist', 'rb') as f:
    manifest = plistlib.load(f)

identity = None
for ident in manifest['BuildIdentities']:
    if ident.get('Info', {}).get('DeviceClass', '') == 'n841ap':
        identity = ident
        break

# Build parameters exactly like pymobiledevice3 restore flow
parameters = {}

# Device values
parameters['ApECID'] = 0x001C15D43C20802E
parameters['ApNonce'] = bytes.fromhex('38b4510c5dbb94ab3ce33a55c3ff45133f9ad1b2e438f1dbc90ce90dba2f6539')
parameters['ApSepNonce'] = bytes.fromhex('42c320e4168699909c463ef6f8ba7e1eb6ff5242')
parameters['ApProductionMode'] = True
parameters['ApSecurityMode'] = True
parameters['ApSupportsImg4'] = True

# Populate from manifest (exactly like populate_tss_request_from_manifest)
key_list = [
    'ApBoardID', 'ApChipID', 'UniqueBuildID', 'Ap,OSLongVersion',
    'Ap,OSReleaseType', 'Ap,ProductType', 'Ap,SDKPlatform', 'Ap,SikaFuse',
    'Ap,Target', 'Ap,TargetType', 'ApSecurityDomain',
    'BMU,BoardID', 'BMU,ChipID', 'BbChipID', 'BbProvisioningManifestKeyHash',
    'BbActivationManifestKeyHash', 'BbCalibrationManifestKeyHash',
    'Ap,ProductMarketingVersion', 'BbFactoryActivationManifestKeyHash',
    'BbFDRSecurityKeyHash', 'BbSkeyId', 'SE,ChipID', 'Savage,ChipID',
    'Savage,PatchEpoch', 'Yonkers,BoardID', 'Yonkers,ChipID',
    'Yonkers,PatchEpoch', 'Rap,BoardID', 'Rap,ChipID', 'Rap,SecurityDomain',
    'Baobab,BoardID', 'Baobab,ChipID', 'Baobab,ManifestEpoch',
    'Baobab,SecurityDomain', 'eUICC,ChipID', 'PearlCertificationRootPub',
    'Timer,BoardID,1', 'Timer,BoardID,2', 'Timer,ChipID,1', 'Timer,ChipID,2',
    'Timer,SecurityDomain,1', 'Timer,SecurityDomain,2', 'Manifest', 'NeRDEpoch',
]

for k in key_list:
    v = identity.get(k)
    if v is not None:
        if isinstance(v, str) and v.startswith('0x'):
            v = int(v, 16)
        parameters[k] = v

# Build request exactly like get_tss_response
tss = TSSRequest()
tss.add_common_tags(parameters)
tss.add_ap_tags(parameters)
tss.add_ap_img4_tags(parameters)

print(f"Request has {len(tss._request)} keys")
comp_keys = [k for k in tss._request.keys() 
             if not k.startswith('@') and not k.startswith('Ap') 
             and k not in ('UniqueBuildID', 'PearlCertificationRootPub')]
print(f"Components: {len(comp_keys)}")
nonce_hex = tss._request.get('ApNonce', b'').hex()
print(f"ApNonce: {nonce_hex[:20]}...")

# Save request for debugging
data = plistlib.dumps(tss._request)
with open('tools/checkm8-dualboot/cache/tss_nonce_request.plist', 'wb') as f:
    f.write(data)
print("Request saved")

print("\nSending TSS request with real device nonce...")
try:
    resp = asyncio.run(tss.send_receive())
    print("SUCCESS!!!")
    # TSSResponse is a dict subclass
    rdata = plistlib.dumps(dict(resp))
    with open('tools/checkm8-dualboot/cache/tss_success_response.plist', 'wb') as f:
        f.write(rdata)
    print(f"Response saved ({len(rdata)} bytes)")
    print(f"Response keys: {sorted(resp.keys())[:15]}")
    # Check for ApImg4Ticket
    if 'ApImg4Ticket' in resp:
        ticket = resp['ApImg4Ticket']
        print(f"\nApImg4Ticket: {len(ticket)} bytes")
        with open('tools/checkm8-dualboot/cache/apticket.der', 'wb') as f:
            f.write(ticket)
        print("Saved to cache/apticket.der")
    # Check iBSS entry
    if 'iBSS' in resp:
        print(f"iBSS entry: {type(resp['iBSS'])}")
except Exception as e:
    print(f"ERROR: {e}")
    import traceback
    traceback.print_exc()
