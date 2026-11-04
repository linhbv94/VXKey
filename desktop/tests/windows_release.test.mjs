import {test} from 'node:test';
import assert from 'node:assert/strict';
import {generateKeyPairSync,randomBytes,sign} from 'node:crypto';
import {mergeWindowsManifest} from '../../scripts/finalize_windows_release.mjs';
function fixture(){
  const {privateKey,publicKey}=generateKeyPairSync('ed25519');
  const id=randomBytes(8),bytes=Buffer.from('MZ synthetic signed installer fixture');
  const publicBytes=Buffer.concat([Buffer.from('Ed'),id,publicKey.export({type:'spki',format:'der'}).subarray(-32)]);
  const signature=sign(null,bytes,privateKey),comment=Buffer.from('fixture only');
  const packet=Buffer.concat([Buffer.from('Ed'),id,signature]);
  const signatureText=Buffer.from(`untrusted comment: fixture\n${packet.toString('base64')}\ntrusted comment: ${comment}\n${sign(null,Buffer.concat([signature,comment]),privateKey).toString('base64')}`).toString('base64');
  return {bytes,signature:signatureText,config:{version:'1.2.3',plugins:{updater:{pubkey:Buffer.from(`untrusted comment: fixture\n${publicBytes.toString('base64')}`).toString('base64')}}},manifest:{version:'1.2.3',notes:'keep notes',platforms:{'darwin-aarch64':{url:'https://example.invalid/mac.tar.gz',signature:'unchanged mac'}}}};
}
test('Windows merge verifies signature and preserves Mac entry and notes',()=>{const f=fixture();const merged=mergeWindowsManifest(f.manifest,f.config,f.bytes,f.signature);assert.deepEqual(merged.platforms['darwin-aarch64'],f.manifest.platforms['darwin-aarch64']);assert.equal(merged.notes,f.manifest.notes);assert.match(merged.platforms['windows-x86_64'].url,/vxkey_1\.2\.3_windows_x64_setup\.exe$/);assert.equal(Object.keys(f.manifest.platforms).length,1);});
test('draft retry replaces only Windows entry',()=>{const f=fixture();f.manifest.platforms['windows-x86_64']={url:'old',signature:'old'};const merged=mergeWindowsManifest(f.manifest,f.config,f.bytes,f.signature);assert.notEqual(merged.platforms['windows-x86_64'].signature,'old');assert.equal(merged.platforms['darwin-aarch64'].signature,'unchanged mac');});
test('version, missing Mac, or unknown platform stop merge',()=>{const f=fixture();assert.throws(()=>mergeWindowsManifest({...f.manifest,version:'9.9.9'},f.config,f.bytes,f.signature));assert.throws(()=>mergeWindowsManifest({...f.manifest,platforms:{}},f.config,f.bytes,f.signature));f.manifest.platforms.linux={};assert.throws(()=>mergeWindowsManifest(f.manifest,f.config,f.bytes,f.signature));});
test('tampered package or another key cannot enter manifest',()=>{const f=fixture(),other=fixture();assert.throws(()=>mergeWindowsManifest(f.manifest,f.config,Buffer.from('tampered'),f.signature));assert.throws(()=>mergeWindowsManifest(f.manifest,f.config,f.bytes,other.signature));});
