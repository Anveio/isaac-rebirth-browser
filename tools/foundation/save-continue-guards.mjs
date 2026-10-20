import assert from 'node:assert/strict';
import {assertPublicRightMovement} from './save-continue-movement.mjs';
import {createHash} from 'node:crypto';
import {readFile,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
const digest=value=>createHash('sha256').update(value).digest('hex');
//The current producer explicitly qualifies its own V2 transport. Historical
//V1 captures keep their archived producer and are never regraded as V2.
export const OWN_PROFILE_SCHEMA=2;
export function readOwnProfilePacket(raw) {
  const packet=JSON.parse(raw);assert.equal(packet.version,OWN_PROFILE_SCHEMA);assert.equal(packet.profiles?.length,3);
  for(const {profile} of packet.profiles) {
    const history=profile?.collectedHistory;
    assert.ok(history&&['sourceReset','ownVersion1Unknown'].includes(history.origin),'Actual V2 collected-history origin is mandatory');
    assert.equal(history.values?.length,347,'Actual V2 collected-history capacity is mandatory');
    assert.ok(history.values.every(value=>typeof value==='boolean'||value===null),'Unknown history remains distinct from known boolean ownership');
    if(history.origin==='sourceReset')assert.ok(history.values.every(value=>typeof value==='boolean'),'Fresh constructor history cannot acquire unknown entries');
  }
  return packet;
}
export async function ownPacket(frame) {
  assert.ok(frame?.ownSave?.store,'Actual own store packet is required');
  const raw=await readFile(frame.ownSave.store.path);assert.equal(digest(raw),frame.ownSave.store.sha256);assert.equal(raw.length,frame.ownSave.store.bytes);
  const packet=readOwnProfilePacket(raw);
  const record=packet.profiles[0].saved;assert.ok(record);return record;
}
export async function validateSaveCapture(frames,directory,caseData) {
  const checks=[];
  const required=async(name,body)=>{try{await body();checks.push({name,status:'pass'});}catch(error){checks.push({name,status:'fail',error:String(error)});}};
  const frame=label=>{const found=frames.filter(row=>row.label===label);assert.equal(found.length,1,label);return found[0];};
  for(const row of frames)await required(row.label+':actual-own-IO',()=>{
    assert.equal(row.ownSave?.io?.protocol,1);assert.equal(row.ownSave.io.declared,true);assert.equal(row.ownSave.io.storageAvailable,true);
    assert.equal(row.ownSave.io.truncated,false);assert.equal(row.ownSave.storeError,null);
    assert.ok(row.ownSave.io.events.every(event=>['pass','absent'].includes(event.status)),'Every real IO failure remains a failure, not missing-file success');
    assert.equal(row.state.saveWritePending,false);assert.equal(row.state.saveReadmissionPending,false);
  });
  await required('Quit-produces-newer-actual-own-save',async()=>{
    const initial=frame('save-run-ready'),prior=frame('save-before-quit'),main=frame('save-main-settled');
    const initialRecord=await ownPacket(initial),record=await ownPacket(main);
    assert.equal(prior.state.player.dead,false);assert.equal(prior.state.paused,true);assert.equal(prior.state.pauseMenu.selection,2);
    assert.ok(main.state.saveRevision>prior.state.saveRevision,'A stored admission packet alone is insufficient');
    assert.ok(record.run.gameUpdateCount>initialRecord.run.gameUpdateCount);
    assert.equal(record.run.gameUpdateCount,prior.state.gameUpdateCount,'Quit must copy its actual paused Game update owner');
    const code=caseData.seed.replaceAll(' ','').toUpperCase();
    assert.match(code,/^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/);
    assert.equal(record.run.runSeed,code.slice(0,4)+' '+code.slice(4),'Actual Saved+EF80 text');assert.equal(record.run.seededRun,true);
    assert.equal(record.run.stage,1);assert.equal(record.context.character,'Isaac');assert.equal(record.context.challengeId,0);assert.equal(record.context.sourceEffectMode,0);
    assert.equal(record.run.currentRoom,record.run.floor.startingDescriptor);assert.equal(record.context.visitedRooms.length,1);
    const footprintWitness = caseData.saveContinueContract?.stationaryBeforeSave === false;
    assert.equal(typeof caseData.saveContinueContract?.stationaryBeforeSave,'boolean','Case must declare its preSave movement domain');
    assert.ok(Array.isArray(record.run.room.backgroundStains));
    if(footprintWitness){
      assert.ok(record.run.room.backgroundStains.length>0,'Moving witness must actually produce preSave footprints');
      assert.ok(record.run.room.backgroundStains.every(stain=>stain.footprint===true&&stain.variant===7),'Only actual owned Blood7 footprints are admitted');
    }else assert.equal(record.run.room.backgroundStains.length,0,'Stationary witness must retain its declared no-paint domain');
    assert.equal(main.state.screen,'main');assert.equal(main.state.mainSelection,1);assert.equal(main.state.savedLoadValid,true);
    assert.ok(main.ownSave.io.events.some(event=>event.operation==='save'&&event.status==='pass'&&event.revision>prior.state.saveRevision&&event.packet?.sha256===main.ownSave.store.sha256),'Actual newer write/readback of the exact stored packet is mandatory');
    assert.ok(main.ownSave.io.events.some(event=>event.operation==='readmit'&&event.status==='pass'&&event.packet?.sha256===main.ownSave.store.sha256),'Actual menu read must consume the actual own bytes');
    for(const row of [initial,prior,main])assert.ok(row.state.bootLifecycle.savedGridSeedSamples.some(sample=>sample.roomRecord===127&&sample.gridIndex===447),'Actual unused row127 projection required');
  });
  await required('same-process-real-Continue-world',async()=>{
    const main=frame('save-main-settled'),resumed=frame('save-continued-playing'),moved=frame('save-continued-coast');const record=await ownPacket(main);
    assert.equal(resumed.state.screen,'playing');assert.equal(resumed.state.runSeedValue,caseData.seedValue);assert.equal(resumed.state.runSeed,caseData.seed.replaceAll(' ','').toUpperCase(),'Actual internal compact run code');
    assert.equal(resumed.state.stage,record.run.stage);assert.equal(resumed.state.currentRoom,record.run.currentRoom);assert.deepEqual(resumed.state.floor,record.run.floor);
    assert.deepEqual(resumed.state.inventory,record.run.inventory);assert.equal(resumed.state.player.hp,record.run.player.hp);assert.equal(resumed.state.player.maxHp,record.run.player.maxHp);
    assert.ok(resumed.state.room.loaded&&resumed.state.room.grid?.cells.length>0);
    assert.ok(Array.isArray(resumed.state.room.backgroundStains),'Actual continued Room paint owner is mandatory');
    assert.equal(resumed.state.room.backgroundStains.length,0,'Room.Init must clear its transient footprint paint before postContinue movement');
    assertPublicRightMovement(resumed.state,moved.state,record.run.floor,record.run.currentRoom);
  });
  await writeFile(join(directory,'save-own-guards.json'),JSON.stringify({schema:1,ownProfileSchema:OWN_PROFILE_SCHEMA,status:checks.every(row=>row.status==='pass')?'pass':'fail',checks,
    scope:'Mandatory actual own-IO and initial-room living Quit/Continue; no Native parity or dead-popup success inferred.'},null,2)+'\n',{flag:'wx'});
  return checks;
}
