// Real Windows helper/pipe checks. Never resume or inject typing during this test.
import {spawn} from 'node:child_process';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {randomUUID} from 'node:crypto';
import assert from 'node:assert/strict';
const executable=resolve(process.argv[2]);
const output=resolve(process.argv[3]);
const child=spawn(executable,['--windows-engine-helper'],{stdio:['pipe','pipe','ignore'],windowsHide:true});
let buffer=Buffer.alloc(0);let pending=[];let frames=[];
child.stdout.on('data',data=>{buffer=Buffer.concat([buffer,data]);while(buffer.length>=4){const n=buffer.readUInt32LE();assert(n<2097153);if(buffer.length<n+4)break;const value=JSON.parse(buffer.subarray(4,n+4));buffer=buffer.subarray(n+4);if(pending.length)pending.shift()(value);else frames.push(value);}});
const next=()=>Promise.race([new Promise(resolve=>frames.length?resolve(frames.shift()):pending.push(resolve)),new Promise((_,reject)=>setTimeout(()=>reject(Error('helper response timeout')),5000))]);
const session=randomUUID();let sequence=0;
async function request(command,extra={}){sequence++;const value={protocol:1,session,sequence,command,...extra};const bytes=Buffer.from(JSON.stringify(value));const head=Buffer.alloc(4);head.writeUInt32LE(bytes.length);child.stdin.write(Buffer.concat([head,bytes]));const response=await next();assert.equal(response.session,session);assert.equal(response.sequence,sequence);assert.equal(response.pid,child.pid);return response;}
const config=JSON.parse(readFileSync(new URL('../../crates/vxkey-runtime/src/defaults/config_example_windows.json',import.meta.url)));
const checks=[];
try {
  let result=await request('bootstrap',{config});assert.equal(result.ok,true);assert.equal(result.running,false);checks.push('fresh_helper_paused');
  const invalid=structuredClone(config);invalid.revision='2';invalid.shortcuts.toggle={kind:'key_chord',modifiers:['meta'],key_code:'KeyV'};
  result=await request('configure',{config:invalid});assert.equal(result.ok,false);assert.equal(result.revision,'1');checks.push('reserved_binding_rejected_without_config_change');
  result=await request('snapshot');assert.equal(result.ok,true);assert.equal(result.running,false);checks.push('control_survives_rejected_config_pause_preserved');
  config.revision='2';config.typing.input_method='vni';config.macros.entries=[{id:randomUUID(),trigger:'vxqa',replacement:'Đã kiểm 🧪'}];
  result=await request('configure',{config});assert.equal(result.ok,true);assert.equal(result.revision,'2');assert.equal(result.running,false);checks.push('vni_macro_config_ack_paused');
  result=await request('pause');assert.equal(result.ok,true);assert.equal(result.running,false);checks.push('explicit_pause_ack');
  await request('shutdown');
  await new Promise((resolve,reject)=>{child.once('exit',code=>code===0?resolve():reject(Error(`helper exit ${code}`)));setTimeout(()=>reject(Error('shutdown timeout')),5000);});checks.push('explicit_shutdown_reaps_helper');
  writeFileSync(output,JSON.stringify({status:'PASS',checks,native_typing:false},null,2));console.log(JSON.stringify({status:'PASS',checks}));
}finally {if(child.exitCode===null)child.kill();}
