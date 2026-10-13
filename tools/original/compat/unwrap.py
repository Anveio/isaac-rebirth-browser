#!/usr/bin/env python3
"""Restore this owner's known Steam-wrapped ELF into a derived working copy.
Only wrapper metadata is parsed; gameplay instructions are never analyzed.
"""
import hashlib
import json
from pathlib import Path
import shutil
import struct
from Crypto.Cipher import AES
from elftools.elf.elffile import ELFFile
root=Path(__file__).resolve().parents[3]
source=root/'references/isaac/linux/depot_250903'
dest=root/'.cache/original-runtime/offline-depot'
manifest=json.loads((source.parent/'manifest.json').read_text())
expected=next(item['sha256'] for item in manifest['files'] if item['path']=='isaac.x64')
raw=(source/'isaac.x64').read_bytes()
if hashlib.sha256(raw).hexdigest()!=expected:raise SystemExit('Source game hash mismatch; restore the verified depot first.')
with (source/'isaac.x64').open('rb') as stream:
    elf=ELFFile(stream);bind=elf.get_section_by_name('.bind');text=elf.get_section_by_name('.text')
    entry=elf.header['e_entry'];entry_offset=entry-bind['sh_addr']
    header_bytes=bind.data()[entry_offset-0xc0:entry_offset]
key=struct.unpack_from('<I',header_bytes)[0];header=bytearray()
for offset in range(4,len(header_bytes),4):
    word=struct.unpack_from('<I',header_bytes,offset)[0];header+=struct.pack('<I',word^key);key=word
if len(header)!=188 or struct.unpack_from('<I',header)[0]!=0xc0dec0de:raise SystemExit('Unsupported wrapper header.')
base=struct.unpack_from('<Q',header,4)[0]
restored_entry=base+struct.unpack_from('<I',header,0x18)[0]
code_offset,code_size=struct.unpack_from('<II',header,0x3c)
if code_offset!=text['sh_offset'] or code_size!=text['sh_size']//16*16:raise SystemExit('Unexpected encrypted code range.')
if struct.unpack_from('<I',header,0x2c)[0]!=250900:raise SystemExit('Unexpected Steam app ID.')
key=bytes(header[0x44:0x64]);iv=AES.new(key,AES.MODE_ECB).decrypt(bytes(header[0x64:0x74]))
ciphertext=bytes(header[0x74:0x84])+raw[code_offset:code_offset+code_size]
plaintext=AES.new(key,AES.MODE_CBC,iv).decrypt(ciphertext)
if AES.new(key,AES.MODE_CBC,iv).encrypt(plaintext)!=ciphertext:raise SystemExit('Payload round-trip failed.')
crt_offset=restored_entry-text['sh_addr']
if plaintext[crt_offset:crt_offset+15]!=bytes.fromhex('31ed4989d15e4889e24883e4f05054'):raise SystemExit('Restored ELF startup ABI validation failed.')
working=bytearray(raw);working[code_offset:code_offset+code_size]=plaintext[:code_size]
struct.pack_into('<Q',working,24,restored_entry)
# No other bytes are permitted to differ.
assert working[:24]==raw[:24] and working[32:code_offset]==raw[32:code_offset] and working[code_offset+code_size:]==raw[code_offset+code_size:]
dest.mkdir(parents=True,exist_ok=True)
path=dest/'isaac.x64'
if path.is_symlink():path.unlink()
path.write_bytes(working);path.chmod(0o755)
for item in source.iterdir():
    if item.name in ('isaac.x64','lib64'):continue
    link=dest/item.name
    if not link.exists():link.symlink_to(item,target_is_directory=item.is_dir())
(dest/'lib64').mkdir(exist_ok=True)
shutil.copyfile(root/'.cache/original-runtime/offline-steam.so',dest/'lib64/libsteam_api.so')
provenance={'sourceSha256':expected,'derivedSha256':hashlib.sha256(working).hexdigest(),'derivedSteamApiSha256':hashlib.sha256((dest/'lib64/libsteam_api.so').read_bytes()).hexdigest(),'originalEntry':hex(entry),'restoredEntry':hex(restored_entry),'wrapperVersion':'0xc0dec0de','payloadRoundTrip':True,'changedRanges':[{'offset':24,'bytes':8,'purpose':'ELF entry point'},{'offset':code_offset,'bytes':code_size,'purpose':'wrapper-decrypted payload'}],'sourceGameUnmodified':True,'source':'licensed-linux-base-depot','exactStateParity':'not-tested'}
(dest/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
print(json.dumps({'offlineReference': str(dest), 'sourceGameUnmodified': True}))
