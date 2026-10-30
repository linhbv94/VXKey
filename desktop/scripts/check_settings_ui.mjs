// VX Settings UI integrity gate v1.0.0. Self-contained in every app repo.
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
const index=process.argv.indexOf('--app-root');
const root=index>=0?resolve(process.argv[index+1]):resolve(dirname(fileURLToPath(import.meta.url)),'..');
const base=resolve(root,'src/components/shared');
const manifest=JSON.parse(readFileSync(resolve(base,'settings_ui_manifest.json'),'utf8'));
if(manifest.version!=='1.0.0')throw Error('Unsupported Settings UI version');
for(const name of ['settings_ui.tsx','settings_ui.css','settings_copy.ts']) {
 const content=readFileSync(resolve(base,name),'utf8').replace(/\r\n/g,'\n');
 const actual=createHash('sha256').update(content,'utf8').digest('hex');
 if(actual!==manifest.files[name])throw Error(`Settings UI drift: ${name}; update canonical component then sync, do not patch vendor separately.`);
}
console.log(`Settings UI ${manifest.version}: source/CSS/copy integrity PASS`);
