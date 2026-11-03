import {readFileSync} from 'node:fs';
import {verifySignature} from './verify_signature.mjs';
const root = new URL('../',import.meta.url);
const config=JSON.parse(readFileSync(new URL('desktop/src-tauri/tauri.conf.json',root)));
const manifest=JSON.parse(readFileSync(new URL('build/release_assets/latest.json',root)));
const asset=manifest.platforms['darwin-aarch64'];
const bytes=readFileSync(new URL('build/release_assets/'+asset.url.split('/').at(-1),root));
verifySignature(bytes,config.plugins.updater.pubkey,asset.signature);
const tamper=Buffer.from(bytes);tamper[Math.floor(tamper.length/2)]^=1;
let rejected=false;try {verifySignature(tamper,config.plugins.updater.pubkey,asset.signature);}catch {rejected=true;}
if(!rejected)throw new Error('Tamper accepted');
console.log('Updater signature and tamper rejection PASS');
