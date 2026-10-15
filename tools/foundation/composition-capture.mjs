// Real DOM/WASM capture from declared boot resources; no native inputs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {writeJsonEvidence} from './browser-evidence.mjs';
import {BrowserProgress} from './composition-progress.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const keyNames = {Return:'Enter',Down:'ArrowDown',Up:'ArrowUp',Left:'ArrowLeft',Right:'ArrowRight',space:'Space'};
const gameplayKeys = new Set(['Right','Left','Up','Down','w','a','s','d','space','e','q']);
const keyCode = key => {
  if (keyNames[key]) return keyNames[key];
  assert.ok(/^[a-z]$/i.test(key),'Undeclared public key '+key);
  return 'Key' + key.toUpperCase();
};
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');

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
  const resetInput = {initialSeed,bootEpochSeed:declaration.bootEpochSeed,resources};
  const resourceBytes = Buffer.byteLength(JSON.stringify(resources));
  const resetBytes = Buffer.byteLength(JSON.stringify(resetInput));
  assert.ok(resourceBytes <= 512 * 1024 && resetBytes <= 512 * 1024 && xml.length <= 64 * 1024);
  await save(join(directory,'declared-reset-input.json'),resetInput);
  const context = await browser.newContext({viewport:{width:960,height:540}});
  let page;
  try {page=await context.newPage();}
  catch (error) {
    let cleanupError;
    try {await context.close();} catch (failure) {cleanupError=String(failure);}
    await save(join(directory,'preflight-allocation-failure.json'),{status:'fail',error:String(error),cleanupError,scope:'No tape/game capture; actual allocation failure retained.'});
    throw error;
  }
  const states = [], preludeStates = [], captures = [], commands = [], inputs = [], errors = [];
  let progress;
  try {progress=await BrowserProgress.create(directory);}
  catch(error){try{await context.close();}catch(failure){errors.push('Context cleanup: '+String(failure));}throw error;}
  let presentation = 0, gameplayPresentation = null, trace = null, traceError;
  let completedTape = false, archiveWritten = false;
  page.on('pageerror',error=>errors.push(String(error)));
  const step = async count => {
    await page.evaluate(count=>window.game.step(count),count);
    commands.push({type:'tick',count});presentation += count;
  };
  const key = async (name,down) => {
    const code = keyCode(name);
    await page.keyboard[down?'down':'up'](keyNames[name] ?? name);
    commands.push({type:'key',code,down});inputs.push({name,code,down,presentation});
  };
  const checkpoint = async label => {
    const captured = await page.evaluate(()=>window.game.capture());
    const bytes = Buffer.from(captured.png,'base64');
    const path = join(directory,label+'.png');await writeFile(path,bytes);
    captures.push({label,presentation,gameplayPresentation,path,sha256:hash(bytes),state:captured.state});
    await progress.append('capture',captures.at(-1));
    return captured.state;
  };
  const press = async (name,settle,label) => {
    const advance = async count => {
      if (label !== 'gameplay-start') {await step(count);return;}
      for (let i=0;i<count;i++) {
        await step(1);
        preludeStates.push({presentation,relativeTitlePresentation:presentation-63,state:await page.evaluate(()=>window.game.snapshot())});
        await progress.append('preludeState',preludeStates.at(-1));
      }
    };
    await key(name,true);await advance(12);await key(name,false);await advance(settle);
    return checkpoint(label);
  };
  try {
    await progress.append('start',{declarationName:declaration.name,declaredResourceHashes,resetInput});
    await page.goto(server.url+'/?manual=1');
    await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(()=>document.body.dataset.ready),'true');
    await page.evaluate(({initialSeed,bootEpochSeed,resources})=>{
      window.game.pause();window.game.reset(initialSeed,{bootEpochSeed,resources,trace:true});
    },resetInput);
    // Native intro is separately declared. The shared public path begins at
    // the existing first-Title boundary, not at an invented common boot tick.
    await step(63);await checkpoint('title');
    await press('Return',72,'profile');await press('Return',72,'main');
    await press('Down',72,'challenge-selected');await press('Return',72,'challenges');
    await press('Return',300,'gameplay-start');
    gameplayPresentation = 0;
    states.push({presentation,gameplayPresentation,state:await page.evaluate(()=>window.game.snapshot())});
    await progress.append('state',states.at(-1));
    if (declaration.initialPhaseOffset) {
      assert.equal(declaration.initialPhaseOffset,1);await step(1);gameplayPresentation++;
      states.push({presentation,gameplayPresentation,state:await page.evaluate(()=>window.game.snapshot())});
      await progress.append('state',states.at(-1));
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
        states.push({presentation,gameplayPresentation,state:await page.evaluate(()=>window.game.snapshot())});
        await progress.append('state',states.at(-1));
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
    try {trace=await page.evaluate(()=>window.game.trace());}
    catch (error) {traceError=String(error);errors.push('Actual trace unavailable: '+traceError);}
    // Archive before validity/coverage/comparison assertions. Failed and
    // partial captures keep their real states, commands and remaining trace.
    try {
      const evidence = await writeJsonEvidence(join(directory,'browser-evidence.json.gz'),{
        schema:1,captureStatus:completedTape?'complete':'partial',states,preludeStates,captures,trace,traceError,
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
      finally {try {await context.close();} catch (error) {errors.push('Actual context cleanup failed: '+String(error));}}
    }
  }
  const validation = [];
  try {
    assert.equal(completedTape,true);assert.equal(archiveWritten,true);assert.deepEqual(errors,[]);assert.ok(trace);
    assert.equal(trace.enabled,true);assert.equal(trace.inputTraceTruncated,false);
    assert.equal(trace.backendStateTruncated,false);assert.equal(trace.backendStateTruncationReason,null);
    assert.deepEqual(trace.commands,commands);assert.equal(trace.initialSeed,initialSeed);
    assert.equal(trace.bootEpochSeed,declaration.bootEpochSeed);
    assert.ok(trace.backendEvents.length>0&&trace.backendEvents.every(event=>event.state));
    assert.equal(states.length,1+declaration.initialPhaseOffset+declaration.actions.reduce((n,a)=>n+a.presentations,0));
    assert.equal(preludeStates.length,312,'Every real run-confirm/admission presentation is retained');
    for (const [label,screen] of [['title','title'],['profile','profile'],['main','main'],['challenge-selected','main'],['challenges','challenge'],['gameplay-start','playing']]) {
      assert.equal(captures.find(c=>c.label===label)?.state.screen,screen,'Actual navigation checkpoint '+label);
    }
    assert.equal(captures.find(c=>c.label==='challenge-selected').state.mainSelection,2);
    assert.equal(captures.find(c=>c.label==='challenges').state.challengeSelection,1);
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
