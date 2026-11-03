import {test} from 'node:test';
import assert from 'node:assert/strict';
import {wrapUpdate} from '../src/services/update_handle.ts';
for (const failing of ['download','install',null]) test('helper stops only after successful install: '+failing,async()=>{
 const events=[]; const step=async(name)=>{events.push(name);if(name===failing)throw new Error(name);};
 const handle=wrapUpdate({version:'1.0.1',close:async()=>{},download:()=>step('download'),install:()=>step('install')},()=>step('prepare_restart'));
 if(failing)await assert.rejects(handle.downloadAndInstall(()=>{}));else await handle.downloadAndInstall(()=>{});
 assert.deepEqual(events,failing==='download'?['download']:failing==='install'?['download','install']:['download','install','prepare_restart']);
});
