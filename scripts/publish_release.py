"""Publish only a verified draft after its successful release build. No key material."""
import json,os,re,subprocess
from pathlib import Path
repo='linhbv94/VXKey';tag=os.environ['RELEASE_TAG'];run_id=os.environ['BUILD_RUN'];assert re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+',tag) and run_id.isdecimal()
def gh(*args):return json.loads(subprocess.check_output(['gh',*args],text=True))
run=gh('api',f'repos/{repo}/actions/runs/{run_id}');assert run['status']=='completed' and run['conclusion']=='success' and run['path']=='.github/workflows/release.yml'
sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
# Tooling recovery may change workflow only; runtime/package inputs must match.
if run['head_sha']!=sha:
 compare=gh('api',f'repos/{repo}/compare/{sha}...{run["head_sha"]}')
 assert compare.get('files') and all(f['filename'].startswith('.github/workflows/') for f in compare['files'])
release=gh('release','view',tag,'--repo',repo,'--json','tagName,isDraft,isPrerelease,assets');assert release['tagName']==tag and release['isDraft'] and not release['isPrerelease']
version=tag[1:];expected={'latest.json',f'vxkey_{version}_macos_silicon.dmg',f'vxkey_{version}_macos_silicon.app.tar.gz'};assert {a['name'] for a in release['assets']}==expected
out=Path('build/release_assets');out.mkdir(parents=True,exist_ok=True)
subprocess.run(['gh','release','download',tag,'--repo',repo,'--dir',str(out)],check=True)
m=json.loads((out/'latest.json').read_text());assert m['version']==version and set(m['platforms'])=={'darwin-aarch64'}
a=m['platforms']['darwin-aarch64'];assert a['url']==f'https://github.com/{repo}/releases/download/{tag}/vxkey_{version}_macos_silicon.app.tar.gz'
subprocess.run(['node','scripts/check_release_signature.mjs'],check=True)
subprocess.run(['gh','release','edit',tag,'--repo',repo,'--draft=false','--latest'],check=True)
print('Published verified draft '+tag)
