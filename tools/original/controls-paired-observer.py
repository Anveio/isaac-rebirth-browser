"""Declared intro/menu/Playing owners for four fixed Controls routes and same-label OS inputs."""
import importlib.util
import configparser
import hashlib
import os
import stat
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
# The fixed bank and its same-label OS route use only these labels. No observed state
# selects an input, menu lifetime, binding, seed or Browser command.
MENU=frozenset((
 'title-ready',
 'profile-ready',
 'main-ready',
 'main-options-selected',
 'options-ready',
 'controls-devices-ready',
 'controls-keyboard-ready',
 'binding-await-key',
 'binding-i-committed',
 'controls-back-saved',
 'options-after-controls',
 'main-after-options',
 'main-new-run-selected',
 'character-ready',
 'seed-entry',
 'character-seed-committed',
 'column1-selected',
 'two-up-columns-saved',
 'escape-bound-to-up',
 'fresh-back-after-escape-bind',
 'down-cleared-up-s-bound',
 'back-refused-sfx187',
 'defaults-selected',
 'defaults-restored',
 'defaults-back-saved',
))
CONTROLS=frozenset((
 'controls-devices-ready',
 'controls-keyboard-ready',
 'binding-await-key',
 'binding-i-committed',
 'controls-back-saved',
 'column1-selected',
 'two-up-columns-saved',
 'escape-bound-to-up',
 'fresh-back-after-escape-bind',
 'down-cleared-up-s-bound',
 'back-refused-sfx187',
 'defaults-selected',
 'defaults-restored',
 'defaults-back-saved',
))
PLAYING=frozenset((
 'gameplay-start',
 'remap-gameplay-start',
 'remapped-i-held',
 'remapped-i-released',
 'old-w-held-no-movement',
 'old-w-released',
 'column0-w-held',
 'column0-w-released',
 'column1-i-held',
 'column1-i-released',
 'default-w-held',
 'default-w-released',
))
INTRO=frozenset((
 'intro',
))
AUDIO187=frozenset((
 'down-cleared-up-s-bound',
 'back-refused-sfx187',
))
if MENU&PLAYING or MENU&INTRO or PLAYING&INTRO or not CONTROLS<=MENU or not AUDIO187<=CONTROLS:raise ValueError('Overlapping/unknown owner scopes')
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
standard=load('controls_standard_actual',ROOT/'tools/original/foundation-paired-observer.py')
menu=load('controls_selected_menu',HERE/'controls-keyboard-observe.py')
playing=load('controls_selected_playing_input',HERE/'controls-playing-keyboard-observe.py')
audio=load('controls_refusal_actual_audio',ROOT/'tools/original/audio-observer.py')
def observe(pid,context):
 label=context.get('label')
 if label not in MENU|PLAYING|INTRO:return{'status':'fail','memoryWritten':False,'error':'Undeclared capture label'}
 actual=standard.observe(pid,context)
 result={'schema':1,'status':actual['status'],'standard':actual,'memoryWritten':False,'executableMappingsRead':False,'label':label,'scope':'Unchanged standard actual menu/player/run plus explicitly selected Controls menu/input owners; final launcher seed proof is mandatory'}
 #Actual independently launched profile options are read after boot at each
 #owned gate. The clock launcher's kept-options/final seed guards stay required.
 path=Path(context['output']).resolve()/'profile/data/binding of isaac rebirth/options.ini'
 fd=os.open(path,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW)
 try:
  info=os.fstat(fd)
  if not stat.S_ISREG(info.st_mode)or info.st_size>64*1024:raise ValueError('Actual options.ini exceeds regular64KiB bound')
  raw=os.read(fd,64*1024+1)
  if len(raw)!=info.st_size:raise ValueError('Actual options.ini read incomplete')
 finally:os.close(fd)
 config=configparser.ConfigParser();config.read_string(raw.decode('utf8'))
 result['startupOptions']={'status':'pass','path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'text':raw.decode('utf8'),'sfxVolume':config.getfloat('Options','SFXVolume'),'musicVolume':config.getfloat('Options','MusicVolume'),'fullscreen':config.getint('Options','Fullscreen')}
 if label in CONTROLS:
  result['controls']=menu.observe(pid,context)
  if result['controls']['status']!='pass':result['status']='fail'
 elif label in PLAYING:
  result['controls']={'status':'skip','reason':'Explicit Playing scope; MenuManager may be destroyed','menuReaderCalled':False}
  result['playingKeyboard']=playing.observe(pid,context)
  if result['playingKeyboard']['status']!='pass'or actual.get('gameplay',{}).get('status')!='pass':result['status']='fail'
 else:result['controls']={'status':'skip','reason':'Explicit intro or standard pre-run menu outside selected Menu9/10 owner scope','menuReaderCalled':False}
 if label in AUDIO187:
  sounds=audio.observe(pid,context);result['audio187']={'status':sounds['status'],'globalSfxVolume':sounds.get('globalSfxVolume'),'systemCounter':sounds.get('systemCounter'),'memoryWritten':sounds.get('memoryWritten'),'executableMappingsRead':sounds.get('executableMappingsRead'),'bytesRead':sounds.get('bytesRead'),'slot':next((slot for slot in sounds.get('slots',[])if slot['id']==187),None)}
  if sounds['status']!='pass'or result['audio187']['slot']is None:result['status']='fail'
 return result
