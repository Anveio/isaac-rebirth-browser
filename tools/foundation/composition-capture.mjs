// Real DOM/WASM capture from declared boot resources; no native inputs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {writeJsonEvidence} from './browser-evidence.mjs';
import {BrowserProgress} from './composition-progress.mjs';
import { evaluateJson } from '../e2e/lib/game.mjs';
import {resetBootElapsed} from './verify.mjs';
import {DECLARED_NATIVE_INTRO} from './composition-wasm-public.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const keyNames = {Return:'Enter',Tab:'Tab',Down:'ArrowDown',Up:'ArrowUp',Left:'ArrowLeft',Right:'ArrowRight',space:'Space'};
const gameplayKeys = new Set(['Right','Left','Up','Down','w','a','s','d','space','e','q']);
const keyCode = key => {
  if (keyNames[key]) return keyNames[key];
  assert.ok(/^[a-z]$/i.test(key),'Undeclared public key '+key);
  return 'Key' + key.toUpperCase();
};
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');

// The same pure selector runs in the page and against retained real trace
// evidence. Capture the public getter ONCE; never clone a whole trace to Node.
export function readTracePart(trace, request) {
  if (request === null) {
    return {keys:Object.keys(trace),fields:Object.fromEntries(Object.entries(trace).filter(([,value])=>!Array.isArray(value))),
      arrays:Object.fromEntries(Object.entries(trace).filter(([,value])=>Array.isArray(value)).map(([key,value])=>[key,value.length]))};
  }
  const {field,index,count}=request;
  if (!Array.isArray(trace[field]) || !Number.isSafeInteger(index) || index<0 ||
      !Number.isSafeInteger(count) || count<1 || count>64)throw Error('Invalid actual trace chunk request');
  return {index,rows:trace[field].slice(index,index+count)};
}

export async function streamCapturedTrace(readPart, directory, expectedCommands) {
  await mkdir(directory,{recursive:false});
  const spool=await BrowserProgress.create(directory);
  const info={arrayCounts:{},commandErrors:[],backendStatesComplete:true};
  let header,transferError;
  try {
    header=await readPart(null);await spool.append('header',header);
    for(const [field,length] of Object.entries(header.arrays)) {
      assert.ok(Number.isSafeInteger(length)&&length>=0,'Actual trace array length');
      let index=0;
      while(index<length) {
        const part=await readPart({field,index,count:field==='commands'?64:1});
        assert.equal(part.index,index);assert.ok(part.rows.length>0&&index+part.rows.length<=length);
        for(const value of part.rows) {
          await spool.append('array:'+field,value);
          if(field==='commands') {
            try {assert.deepEqual(value,expectedCommands[index]);}
            catch(error){info.commandErrors.push({index,error:String(error)});}
          }
          if(field==='backendEvents')info.backendStatesComplete&&=Boolean(value?.state);
          index++;
        }
        info.arrayCounts[field]=index;
      }
      info.arrayCounts[field]=index;
    }
    await spool.append('finish',{status:'complete',info});
  } catch(error) {
    transferError=String(error);
    await spool.append('failure',{error:transferError,info});
  } finally {await spool.close(transferError?'partial':'closed');}
  const trace=header?Object.fromEntries(header.keys.map(key=>[key,
    Object.hasOwn(header.arrays,key)?spool.values('array:'+key):header.fields[key]])):null;
  return {trace,info,error:transferError};
}

export async function captureComposition(browser, server, declaration, directory) {
  await mkdir(directory, {recursive:true});
  const xml = await readFile(declaration.fixture.authoredResourcePath);
  const resources = {challengesXml:xml.toString('utf8')};
  const declaredResourceHashes = {challengesXml:hash(xml)};
  if (declaration.fixture.specialRoomsPath) {
    const stb = await readFile(declaration.fixture.specialRoomsPath);
    if (declaration.fixture.specialRoomsSha256) assert.equal(hash(stb), declaration.fixture.specialRoomsSha256);
    resources.specialRooms = Array.from(stb);
    declaredResourceHashes.specialRooms = hash(stb);
  }
  const initialSeed = 1; // Independently declared reset placeholder, never native admitted seed.
  const audioPreferences = {sfxVolume: declaration.pocketInit || declaration.pocketCollection || declaration.commonSku?.schema === 'source-public-common-sku-v1' ? declaration.sfxVolume : 0, musicVolume: 0};
  assert.ok([audioPreferences.sfxVolume, audioPreferences.musicVolume].every(value => Number.isFinite(value) && value>=0 && value<=1));
  const resetInput = {initialSeed,bootEpochSeed:declaration.bootEpochSeed,resources,audioPreferences,fullscreenPreferences:{fullscreen:false},
    bootElapsed:resetBootElapsed(DECLARED_NATIVE_INTRO)};
  const resourceBytes = Buffer.byteLength(JSON.stringify(resources));
  const resetBytes = Buffer.byteLength(JSON.stringify(resetInput));
  assert.ok(resourceBytes <= 512 * 1024 && resetBytes <= 512 * 1024 && xml.length <= 64 * 1024);
  //Retain the exact executing producer bytes before reset. Later legitimate
  //canonical edits cannot invalidate an immutable capture's protocol proof.
  const captureSourceBytes = await readFile(new URL(import.meta.url));
  await writeFile(join(directory,'composition-capture-source.mjs'),captureSourceBytes,{flag:'wx'});
  await save(join(directory,'declared-reset-input.json'),resetInput);
  const startupRecord = {schema:1,protocol:'declared-preboot-audio-fullscreen-v1',phase:'before-reset',resetInput,
    captureSource:{path:fileURLToPath(import.meta.url),sha256:hash(captureSourceBytes),archive:'composition-capture-source.mjs'}};
  await save(join(directory,'declared-startup-inputs.json'),startupRecord);
  const context = await browser.newContext({viewport:{width:960,height:540}});
  let page;
  try {page=await context.newPage();}
  catch (error) {
    let cleanupError;
    try {await context.close();} catch (failure) {cleanupError=String(failure);}
    await save(join(directory,'preflight-allocation-failure.json'),{status:'fail',error:String(error),cleanupError,scope:'No tape/game capture; actual allocation failure retained.'});
    throw error;
  }
  const captureScreens = new Map(), commands = [], inputs = [], errors = [];
  let stateCount = 0, preludeStateCount = 0, contextClosed = false;
  let progress;
  try {progress=await BrowserProgress.create(directory);}
  catch(error){try{await context.close();}catch(failure){errors.push('Context cleanup: '+String(failure));}throw error;}
  let presentation = 0, gameplayPresentation = null, trace = null, traceError, traceInfo;
  let completedTape = false, archiveWritten = false;
  page.on('pageerror',error=>errors.push(String(error)));
  const step = async count => {
    await page.evaluate(count=>{window.game.step(count);},count);
    commands.push({type:'tick',count});presentation += count;
  };
  const key = async (name,down) => {
    const code = keyCode(name);
    await page.keyboard[down?'down':'up'](keyNames[name] ?? name);
    commands.push({type:'key',code,down});inputs.push({name,code,down,presentation});
  };
  const checkpoint = async label => {
    const captured = await evaluateJson(page, ()=>window.game.capture());
    const bytes = Buffer.from(captured.png,'base64');
    const path = join(directory,label+'.png');await writeFile(path,bytes);
    if(!captureScreens.has(label))captureScreens.set(label,{screen:captured.state.screen,mainSelection:captured.state.mainSelection,challengeSelection:captured.state.challengeSelection});
    await progress.append('capture',{label,presentation,gameplayPresentation,path,sha256:hash(bytes),state:captured.state});
    return captured.state;
  };
  const press = async (name,settle,label) => {
    const advance = async count => {
      if (label !== 'gameplay-start') {await step(count);return;}
      for (let i=0;i<count;i++) {
        await step(1);
        await progress.append('preludeState',{presentation,relativeTitlePresentation:presentation-63,state:await evaluateJson(page, ()=>window.game.snapshot())});
        preludeStateCount++;
      }
    };
    await key(name,true);await advance(12);await key(name,false);await advance(settle);
    return checkpoint(label);
  };
  try {
    await progress.append('start',{declarationName:declaration.name,declaredResourceHashes,resetInput,startupRecord});
    await page.goto(server.url+'/?manual=1');
    await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(()=>document.body.dataset.ready),'true');
    await page.evaluate(({initialSeed,bootEpochSeed,resources,audioPreferences,fullscreenPreferences,bootElapsed})=>{
      window.game.pause();window.game.reset(initialSeed,{bootEpochSeed,resources,audioPreferences,fullscreenPreferences,bootElapsed,trace:true});
    },resetInput);
    // Native intro is separately declared. The shared public path begins at
    // the existing first-Title boundary, not at an invented common boot tick.
    await step(63);await checkpoint('title');
    await press('Return',72,'profile');await press('Return',72,'main');
    const seededPocket = Boolean((declaration.pocketInit || declaration.pocketCollection || declaration.commonSku?.schema === 'source-public-common-sku-v1')
      && declaration.publicMenuRoute === 'seeded-normal');
    if (seededPocket) {
      assert.match(declaration.publicSeedCode, /^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/);
      await press('Return',72,'character');await press('Tab',108,'seed-entry');
      for (const text of declaration.publicSeedCode) {
        const code = /^[0-9]$/.test(text) ? 'Digit' + text : keyCode(text);
        await page.keyboard.down(text);commands.push({type:'text',text});
        inputs.push({name:text,code,down:true,presentation,sourceText:true});
        await step(12);await page.keyboard.up(text);
        inputs.push({name:text,code,down:false,presentation,sourceText:true});await step(6);
      }
      assert.equal((await evaluateJson(page, ()=>window.game.snapshot())).screen,'seed');
      await press('Return',72,'character-seeded');
    } else {
      await press('Down',72,'challenge-selected');await press('Return',72,'challenges');
    }
    const entered = await press('Return',300,'gameplay-start');
    if (seededPocket) {
      assert.equal(entered.runSeedValue,declaration.publicRunSeed);
      assert.equal(entered.runSeed.replaceAll(' ',''),declaration.publicSeedCode);
    }
    gameplayPresentation = 0;
    await progress.append('state',{presentation,gameplayPresentation,state:await evaluateJson(page, ()=>window.game.snapshot())});
    stateCount++;
    if (declaration.initialPhaseOffset) {
      assert.equal(declaration.initialPhaseOffset,1);await step(1);gameplayPresentation++;
      await progress.append('state',{presentation,gameplayPresentation,state:await evaluateJson(page, ()=>window.game.snapshot())});
      stateCount++;
    }
    let held = new Set();
    for (const action of declaration.actions) {
      const requested = new Set(action.keys);
      assert.equal(requested.size,action.keys.length);
      assert.ok([...requested].every(key=>gameplayKeys.has(key)),'Gameplay keys must match the declared public grammar');
      assert.ok(Number.isInteger(action.presentations)&&action.presentations>=1&&action.presentations<=400);
      for (const name of [...held].filter(name=>!requested.has(name)).sort()) await key(name,false);
      for (const name of [...requested].filter(name=>!held.has(name)).sort()) await key(name,true);
      for (let i=0;i<action.presentations;i++) {
        await step(1);gameplayPresentation++;
        await progress.append('state',{presentation,gameplayPresentation,state:await evaluateJson(page, ()=>window.game.snapshot())});
        stateCount++;
        if ([3,4].includes(i+1)) await checkpoint(action.label+'-observe-step'+String(i+1));
      }
      await checkpoint(action.label);held=requested;
    }
    for (const name of [...held].sort()) await key(name,false);
    completedTape = true;
  } catch (error) {
    errors.push(String(error));
  } finally {
    // Commit all actual observations BEFORE transferring the final trace.
    // This changes only controller I/O; no virtual tick/input is added.
    try {
      await progress.append('beforeTrace',{completedTape,presentation,gameplayPresentation,commands,inputs,errors});
      await progress.flush();
    } catch(error) {errors.push('Actual committed journal unavailable before trace: '+String(error));}
    let traceHandle;
    try {
      traceHandle=await page.evaluateHandle(()=>window.game.trace());
      const transferred=await streamCapturedTrace(request=>traceHandle.evaluate(readTracePart,request),
        join(directory,'trace'),commands);
      trace=transferred.trace;traceInfo=transferred.info;traceError=transferred.error;
      if(traceError)errors.push('Actual trace unavailable: '+traceError);
    } catch (error) {traceError=String(error);errors.push('Actual trace unavailable: '+traceError);}
    finally {if(traceHandle)await traceHandle.dispose().catch(error=>errors.push('Actual trace handle cleanup failed: '+String(error)));}
    // All page-dependent evidence is collected. Release Chromium before the
    // final archive; replay the fully closed journal instead of retaining a
    // second full observation history in this Node process.
    try {await context.close();contextClosed=true;}
    catch(error){errors.push('Actual context cleanup failed: '+String(error));}
    try {await progress.closeForRead();}
    catch(error){errors.push('Actual committed journal close failed: '+String(error));}
    // Archive before validity/coverage/comparison assertions. Failed and
    // partial captures keep their real states, commands and remaining trace.
    try {
      const evidence = await writeJsonEvidence(join(directory,'browser-evidence.json.gz'),{
        schema:1,startupProtocol:startupRecord.protocol,startupRecord,captureStatus:completedTape?'complete':'partial',
        states:progress.values('state'),preludeStates:progress.values('preludeState'),captures:progress.values('capture'),trace,traceError,
        expectedCommands:commands,actualDomInputs:inputs,errors,declaredResourceHashes,
        independentInitialSeed:initialSeed,declaredBootEpochSeed:declaration.bootEpochSeed,
        declaredResourceJsonBytes:resourceBytes,
        declaredFullResetJsonBytes:resetBytes,
      });
      await save(join(directory,'browser-evidence-meta.json'),evidence);
      await progress.append('archiveComplete',{evidence,completedTape,traceError});
      archiveWritten = true;
    } catch(error) {errors.push('Actual final archive unavailable: '+String(error));
    } finally {
      try {await progress.close(completedTape&&!traceError&&archiveWritten&&!errors.length?'closed':'partial');}
      catch(error){errors.push('Actual committed journal cleanup failed: '+String(error));}
      finally {if(!contextClosed)try {await context.close();contextClosed=true;} catch (error) {errors.push('Actual context cleanup failed: '+String(error));}}
    }
  }
  const validation = [];
  try {
    assert.equal(completedTape,true);assert.equal(archiveWritten,true);assert.deepEqual(errors,[]);assert.ok(trace);
    assert.equal(trace.enabled,true);assert.equal(trace.inputTraceTruncated,false);
    assert.equal(trace.backendStateTruncated,false);assert.equal(trace.backendStateTruncationReason,null);
    assert.equal(traceInfo.arrayCounts.commands,commands.length);assert.deepEqual(traceInfo.commandErrors,[]);
    assert.equal(trace.initialSeed,initialSeed);
    assert.equal(trace.bootEpochSeed,declaration.bootEpochSeed);
    assert.deepEqual(trace.bootAudioPreferences,resetInput.audioPreferences);
    assert.deepEqual(trace.bootFullscreenPreferences,resetInput.fullscreenPreferences);
    assert.ok(traceInfo.arrayCounts.backendEvents>0&&traceInfo.backendStatesComplete);
    assert.equal(stateCount,1+declaration.initialPhaseOffset+declaration.actions.reduce((n,a)=>n+a.presentations,0));
    assert.equal(preludeStateCount,312,'Every real run-confirm/admission presentation is retained');
    const normalPocket = Boolean((declaration.pocketInit || declaration.pocketCollection || declaration.commonSku?.schema === 'source-public-common-sku-v1')
      && declaration.publicMenuRoute === 'seeded-normal');
    const checkpoints = normalPocket
      ? [['title','title'],['profile','profile'],['main','main'],['character','character'],
        ['seed-entry','seed'],['character-seeded','character'],['gameplay-start','playing']]
      : [['title','title'],['profile','profile'],['main','main'],['challenge-selected','main'],['challenges','challenge'],['gameplay-start','playing']];
    for (const [label,screen] of checkpoints) {
      assert.equal(captureScreens.get(label)?.screen,screen,'Actual navigation checkpoint '+label);
    }
    if (!normalPocket) {
      assert.equal(captureScreens.get('challenge-selected').mainSelection,2);
      assert.equal(captureScreens.get('challenges').challengeSelection,1);
    }
    assert.equal(hash(await readFile(declaration.fixture.authoredResourcePath)),declaredResourceHashes.challengesXml);
    if (resources.specialRooms) assert.equal(hash(await readFile(declaration.fixture.specialRoomsPath)),declaredResourceHashes.specialRooms);
    validation.push({field:'actual public input/backend/resource readback',passed:true});
  } catch (error) {validation.push({field:'actual public input/backend/resource readback',passed:false,error:String(error)});}
  const result = {schema:1,status:validation.every(check=>check.passed)?'pass':'fail',validation,
    scope:'Actual production DOM/WASM capture only. Native/generated seed, cache/grants, actor family/contact/persistence and incomplete removal coverage require the independent comparator; capture success is not functional acceptance.',
    declaredResourceHashes,resourceBytes,resetBytes,completedTape,errors};
  await save(join(directory,'capture-report.json'),result);
  return result;
}
