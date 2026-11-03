raise SystemExit('Retired: background VXKey does not register an Input Source. See docs/spec/09_background_driver.md')

"""Register only the installed VXKey; remove only workspace duplicate LS records."""
from pathlib import Path
import json,subprocess
root=Path(__file__).resolve().parents[1]
lsr='/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister'
target=Path.home()/'Library/Input Methods/VXKeyInput.app'
receipt=json.loads((root/'build/native_activation/install_receipt.json').read_text())
assert receipt['status']=='installed' and receipt['target']==str(target)
subprocess.run(['codesign','--verify','--deep','--strict',str(target)],check=True,capture_output=True)
removed=[]
for line in subprocess.check_output([lsr,'-dump'],text=True).splitlines():
    if not line.startswith('path:'):continue
    path=Path(line.split(':',1)[1].strip().split(' (0x',1)[0])
    if path.name=='VXKeyInput.app' and path.is_relative_to(root/'build'):
        subprocess.run([lsr,'-u',str(path)],check=True);removed.append(str(path))
subprocess.run([lsr,'-f',str(target)],check=True)
control=root/'build/native_activation/source_control'
subprocess.run([str(control),'register'],check=True)
state=json.loads(subprocess.check_output([str(control),'status'],text=True))
assert state['launchservices_app']==str(target), 'LS still resolves an uninstalled build copy'
report={'unregistered_workspace_duplicates':removed,'installed_target':str(target),'state':state}
(root/'build/native_activation/registration_cleanup.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
