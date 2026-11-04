// Validate the Windows installer and preserve the Mac draft. Never publishes.
import {readFileSync,writeFileSync,readdirSync,copyFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {verifySignature} from './verify_signature.mjs';
export function mergeWindowsManifest(existing,config,bytes,signature) {
  const version=config.version;
  assert.match(version,/^[0-9]+\.[0-9]+\.[0-9]+$/);
  assert.equal(existing.version,version);
  assert.ok(existing.platforms?.['darwin-aarch64'],'Mac draft must exist first');
  assert.ok(Object.keys(existing.platforms).every(key=>['darwin-aarch64','windows-x86_64'].includes(key)),'Unknown platform must not be dropped');
  verifySignature(bytes,config.plugins.updater.pubkey,signature);
  const tampered=Buffer.from(bytes);tampered[Math.floor(bytes.length/2)]^=1;
  assert.throws(()=>verifySignature(tampered,config.plugins.updater.pubkey,signature));
  const merged=structuredClone(existing);
  merged.platforms['windows-x86_64']={signature,url:`https://github.com/linhbv94/VXKey/releases/download/v${version}/vxkey_${version}_windows_x64_setup.exe`};
  assert.deepEqual(merged.platforms['darwin-aarch64'],existing.platforms['darwin-aarch64']);
  return merged;
}
if(process.argv[1] && resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const root=resolve(fileURLToPath(new URL('..',import.meta.url)));
  const out=resolve(process.argv[2]);
  const existing=JSON.parse(readFileSync(process.argv[3],'utf8').replace(/^\uFEFF/,''));
  const config=JSON.parse(readFileSync(resolve(root,'desktop/src-tauri/tauri.conf.json')));
  const bundle=resolve(root,'desktop/src-tauri/target/release/bundle/nsis');
  const files=readdirSync(bundle).filter(name=>name.endsWith('-setup.exe') && name.startsWith('VXKey_'));
  assert.equal(files.length,1,'Expected one production installer, excluding Test');
  const bytes=readFileSync(resolve(bundle,files[0]));
  const signature=readFileSync(resolve(bundle,files[0]+'.sig'),'utf8').trim();
  const manifest=mergeWindowsManifest(existing,config,bytes,signature);
  mkdirSync(out,{recursive:true});
  const name=`vxkey_${config.version}_windows_x64_setup.exe`;
  copyFileSync(resolve(bundle,files[0]),resolve(out,name));
  writeFileSync(resolve(out,'latest.json'),JSON.stringify(manifest,null,2)+'\n');
  console.log(`Windows updater signature/tamper PASS; preserved Mac platform; prepared ${name}`);
}
