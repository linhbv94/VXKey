import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

root=Path(__file__).resolve().parents[2]/'xfunput'
env=dict(os.environ,DEVELOPER_DIR='/Library/Developer/CommandLineTools')
results={}
with tempfile.TemporaryDirectory(prefix='vxkey_app_policy_') as temp:
    work=Path(temp)
    def run(args):
        p=subprocess.run(args,cwd=root,env=env,capture_output=True,text=True,timeout=90)
        if p.returncode: raise RuntimeError(str(args)+'\n'+p.stderr[:2500]+'\n'+p.stdout[:1000])
        return p.stdout
    guard=(root/'crates/funput-ffi/src/abi/guard.rs').read_text()
    begin=guard.index('pub(crate) fn safe<')
    body=guard.index('{',begin);end=body+1;depth=1
    while depth:
        depth+=(guard[end]=='{')-(guard[end]=='}');end+=1
    shim=work/'app_language_tests.rs'
    shim.write_text('mod abi {use std::panic::{AssertUnwindSafe,catch_unwind};\n'+guard[begin:end]+'}\n#[path='+json.dumps(str(root/'crates/funput-ffi/src/app_language/mod.rs'))+'] mod app_language;\n',encoding='utf-8')
    binary=work/'app_language_tests'
    run(['rustc','+stable','--edition=2024','--test',str(shim),'-o',str(binary)])
    output=run([str(binary)])
    summary=next(line for line in output.splitlines() if line.startswith('test result:'))
    results['actual_app_language_module_tests']={'result':summary,'coverage':'Actual app_language source module and exact safe guard function; isolated rustc harness, not full Cargo/Swift integration.'}
    run(['rustc','+stable','--edition=2024','--crate-type=rlib','--crate-name=funput_core','crates/funput-core/src/lib.rs','--out-dir',str(work)])
    run(['rustc','+stable','--edition=2024','--crate-type=rlib','--crate-name=funput_engine','crates/funput-engine/src/lib.rs','--extern','funput_core='+str(work/'libfunput_core.rlib'),'-L','dependency='+str(work),'--out-dir',str(work)])
    harness=work/'english_macro.rs'
    harness.write_text('''
use funput_engine::{Engine,Action};
fn compose(english_shortcuts:bool)->(String,bool){
 let mut e=Engine::new();e.set_enabled(false);e.add_shortcut("vn","Việt Nam");
 e.update_config(|c|c.shortcuts_in_english=english_shortcuts);
 let mut text=String::new();let mut expanded=false;
 for c in "vn ".chars(){let r=e.process_char(c);if r.action==Action::None{text.push(c);}else{expanded=true;for _ in 0..r.backspace{text.pop();}text.push_str(&r.output);}}
 (text,expanded)
}
fn main(){
 let on=compose(true);assert_eq!(on,("Việt Nam ".to_string(),true));
 let off=compose(false);assert_eq!(off,("vn ".to_string(),false));
 println!("English mode with macro enabled: Việt Nam ; macro disabled: vn ; both controls passed");
}
''',encoding='utf-8')
    binary=work/'english_macro'
    run(['rustc','+stable','--edition=2024',str(harness),'--extern','funput_engine='+str(work/'libfunput_engine.rlib'),'-L','dependency='+str(work),'-o',str(binary)])
    results['english_mode_is_not_hard_bypass']={'result':run([str(binary)]).strip(),'runtime_keyboard_injection':False}

files=['platforms/macos/Funput/Model/AppLanguageMemory.swift','platforms/macos/Funput/Model/AppSettings+VietnameseOverride.swift','platforms/macos/Funput/IME/FunputInputController.swift','platforms/macos/Funput/IME/FunputInputController+Composition.swift','platforms/macos/Funput/IME/FunputInputController+English.swift','platforms/macos/Funput/Model/AppSettingsKeys.swift','platforms/macos/Funput/Convert/Platform/ConvertPlatform.swift','crates/funput-ffi/src/app_language/mod.rs','crates/funput-ffi/src/app_language/focus.rs','crates/funput-ffi/src/app_language/memory.rs','crates/funput-engine/src/model/config.rs']
evidence={'date':'2026-10-27','head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'scope':'Source audit and isolated Rust source module tests only. No live app/exclusion GUI/pasteboard read/write/keyboard injection/permissions/install. Clipboard scope remains design.','toolchain':run(['rustc','+stable','-V']).strip(),'results':results,'source_sha256':{file:hashlib.sha256((root/file).read_bytes()).hexdigest() for file in files}}
Path(__file__).resolve().with_name('app_exclusion_evidence_probe.json').write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(results,ensure_ascii=False))
