#!/usr/bin/env python3
"""Generate typed offline implementations from public Steamworks 1.32 ABI headers.
No original gameplay code is inspected or translated.
"""
from pathlib import Path
import argparse
import re
import subprocess
root = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--headers', type=Path, default=root / '.cache/steam-sdk-132/public/steam')
parser.add_argument('--output', type=Path, default=root / '.cache/original-runtime/offline-steam.cpp')
args = parser.parse_args()
headers = args.headers
out = args.output
base = Path(__file__).with_name('runtime.cpp.in').read_text()
preprocessed = subprocess.check_output(["clang++", "-E", "-P", "-x", "c++", "-I", str(headers), "-include", "steam_api.h", "/dev/null"], text=True)
classes=[]
for name,file in [('User','isteamuser.h'),('Utils','isteamutils.h'),('Stats','isteamuserstats.h'),('Storage','isteamremotestorage.h')]:
    source=headers.joinpath(file).read_text()
    source=re.sub(r'/\*.*?\*/|//[^\n]*','',source,flags=re.S)
    interface={'User':'ISteamUser','Utils':'ISteamUtils','Stats':'ISteamUserStats','Storage':'ISteamRemoteStorage'}[name]
    section=re.search(r'class '+interface+r'\s*\{(.*?)\n\};',preprocessed,re.S).group(1)
    methods=[]
    for m in re.finditer(r'virtual\s+(.+?)\s*=\s*0\s*;',section,re.S):
        sig=' '.join(m.group(1).split())
        fn=re.search(r'(\w+)\s*\(',sig).group(1)
        typ=sig[:sig.index(fn)].strip()
        params=sig[sig.index('(')+1:sig.rindex(')')]
        body=[]
        for parameter in params.split(','):
            pointer=re.search(r'^(bool|int|int32|uint32|float|double)\s*\*\s*(\w+)',parameter.strip())
            if pointer:body.append(f'if ({pointer[2]}) *{pointer[2]} = 0;')
        special={
          ('User','GetSteamID'):'return CSteamID(uint64(0));',
          ('User','BLoggedOn'):'return false;',
          ('User','GetHSteamUser'):'return 0;',
          ('Utils','GetAppID'):'return 250900;',
          ('Utils','GetSteamUILanguage'):'return "english";',
          ('Utils','GetIPCountry'):'return "";',
          ('Stats','RequestCurrentStats'):'pendingReceived = true; return true;',
          ('Stats','StoreStats'):'return storeStats();',
          ('Stats','GetStat'):'return getStat(pchName,pData);',
          ('Stats','GetAchievement'):'return getAchievement(pchName,pbAchieved);',
          ('Stats','GetAchievementAndUnlockTime'):'if(punUnlockTime)*punUnlockTime=0; return getAchievement(pchName,pbAchieved);',
          ('Stats','SetStat'):f'return setStat(pchName,{"fData" if "float fData" in params else "nData"});',
          ('Stats','SetAchievement'):'return setAchievement(pchName,true);',
          ('Stats','ClearAchievement'):'return setAchievement(pchName,false);',
          ('Stats','ResetAllStats'):'return resetStats(bAchievementsToo);',
          ('Stats','GetNumAchievements'):'return localAchievements.size();',
          ('Stats','GetAchievementName'):'if(iAchievement>=localAchievements.size())return ""; auto entry=localAchievements.begin(); std::advance(entry,iAchievement); return entry->first.c_str();',
          ('Storage','FileRead'):'return localRead(pchFile,pvData,cubDataToRead);',
          ('Storage','FileWrite'):'return localWrite(pchFile,pvData,cubData);',
          ('Storage','FileExists'):'return localExists(pchFile);',
          ('Storage','FilePersisted'):'return localExists(pchFile);',
          ('Storage','GetFileSize'):'return localSize(pchFile);',
          ('Storage','FileDelete'):'return localDelete(pchFile);',
          ('Storage','GetQuota'):'if(pnTotalBytes)*pnTotalBytes=1073741824; if(puAvailableBytes)*puAvailableBytes=1073741824; return true;',
        }.get((name,fn))
        if special is not None:
            body = [special]
        else:
            body.append('return;' if typ=='void' else 'return {};')
        methods.append(sig+' override { '+' '.join(body)+' }')
    classes.append('class Offline'+name+' final : public '+interface+' { public:\n'+'\n'.join(methods)+'\n};\n')
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(base.replace('/* GENERATED_INTERFACES */','\n'.join(classes)))
print(out)
