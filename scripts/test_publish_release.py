"""Offline publish gate fixtures. Every subprocess/GitHub operation is mocked."""
import contextlib, io, json, os, runpy, tempfile, unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT = Path(__file__).with_name('publish_release.py')

class PublishGateTests(unittest.TestCase):
    def scenario(self, windows=False, request_windows=None, bad_head=False, bad_url=False, failed=False):
        version='1.2.3'; tag='v'+version; sha='a'*40
        names=['latest.json',f'vxkey_{version}_macos_silicon.dmg',f'vxkey_{version}_macos_silicon.app.tar.gz']
        platforms={'darwin-aarch64':{'signature':'fixture','url':f'https://github.com/linhbv94/VXKey/releases/download/{tag}/'+names[2]}}
        if windows:
            names.append(f'vxkey_{version}_windows_x64_setup.exe')
            platforms['windows-x86_64']={'signature':'fixture','url':('bad' if bad_url else f'https://github.com/linhbv94/VXKey/releases/download/{tag}/'+names[3])}
        calls=[]
        def output(args,**kwargs):
            if args[:2]==['git','rev-parse']:return sha
            if args[:2]==['gh','api']:
                win=args[-1].endswith('/22')
                return json.dumps({'status':'completed','conclusion':'failure' if failed else 'success','path':'.github/workflows/release_windows.yml' if win else '.github/workflows/release.yml','head_sha':'b'*40 if win and bad_head else sha})
            if args[:3]==['gh','release','view']:return json.dumps({'tagName':tag,'isDraft':True,'isPrerelease':False,'assets':[{'name':n} for n in names]})
            raise AssertionError(args)
        def run(args,**kwargs):
            calls.append(args)
            if args[:3]==['gh','release','download']:
                path=Path('build/release_assets');path.mkdir(parents=True,exist_ok=True)
                (path/'latest.json').write_text(json.dumps({'version':version,'platforms':platforms}))
        env={'RELEASE_TAG':tag,'BUILD_RUN':'11','WINDOWS_BUILD_RUN':'22' if (windows if request_windows is None else request_windows) else ''}
        previous=Path.cwd()
        with tempfile.TemporaryDirectory(prefix='vxkey_publish_gate_') as folder:
            try:
                os.chdir(folder)
                with patch.dict(os.environ,env),patch('subprocess.check_output',side_effect=output),patch('subprocess.run',side_effect=run),contextlib.redirect_stdout(io.StringIO()):
                    runpy.run_path(str(SCRIPT))
            finally:os.chdir(previous)
        self.assertTrue(any(c[:3]==['gh','release','edit'] for c in calls))
    def test_mac_only_unchanged(self):self.scenario()
    def test_combined_requires_both_verified_runs(self):self.scenario(windows=True)
    def test_windows_assets_without_windows_run_rejected(self):
        with self.assertRaises(AssertionError):self.scenario(windows=True,request_windows=False)
    def test_wrong_windows_source_rejected(self):
        with self.assertRaises(AssertionError):self.scenario(windows=True,bad_head=True)
    def test_wrong_windows_url_rejected(self):
        with self.assertRaises(AssertionError):self.scenario(windows=True,bad_url=True)
    def test_failed_build_rejected(self):
        with self.assertRaises(AssertionError):self.scenario(failed=True)

if __name__=='__main__':unittest.main()
