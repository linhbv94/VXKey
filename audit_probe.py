import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2] / 'xfunput'
env = dict(os.environ, DEVELOPER_DIR='/Library/Developer/CommandLineTools')
results = {}
with tempfile.TemporaryDirectory(prefix='vxkey_audit_') as temp:
    work = Path(temp).resolve()
    def run(args, timeout=90):
        p = subprocess.run(args, cwd=root, env=env, capture_output=True, text=True, timeout=timeout)
        if p.returncode: raise RuntimeError(str(args) + '\n' + p.stderr[:2500] + '\n' + p.stdout[:1000])
        return p.stdout
    run(['rustc', '+stable', '--crate-name', 'funput_core', '--crate-type', 'rlib', '--edition=2024', 'crates/funput-core/src/lib.rs', '--out-dir', str(work)])
    run(['rustc', '+stable', '--crate-name', 'funput_engine', '--crate-type', 'rlib', '--edition=2024', 'crates/funput-engine/src/lib.rs', '--extern', 'funput_core=' + str(work/'libfunput_core.rlib'), '-L', 'dependency=' + str(work), '--out-dir', str(work)])
    harness = work/'engine_probe.rs'
    harness.write_text('''
use funput_core::{InputMethod, ToneStyle, apply};
use funput_engine::{Engine, Action};
fn compose(raw: &str, method: InputMethod) -> String {
    let mut e=Engine::new(); e.set_method(method);e.update_config(|c|c.tone_style=ToneStyle::Traditional);
    let mut text=String::new();
    for c in raw.chars() {
        let r=e.process_char(c);
        if r.action==Action::None {text.push(c)}
        else {for _ in 0..r.backspace {text.pop();} text.push_str(&r.output);}
    } text
}
fn main() {
    let cases=[("tieesng vieejt",InputMethod::Telex,"tiếng việt"),("xin chao2",InputMethod::Vni,"xin chào"),("dd",InputMethod::Telex,"đ"),("aas",InputMethod::Telex,"ấ"),("hoaf",InputMethod::Telex,"hòa")];
    for (raw,method,expected) in cases {assert_eq!(compose(raw,method),expected,"{raw}");}
    assert_eq!(apply("a",'1',InputMethod::Vni,ToneStyle::Traditional).text,"á");
    let mut e=Engine::new();e.set_enabled(false);assert_eq!(e.process_char('a').action,Action::None);
    e.set_enabled(true);e.process_char('a');e.set_method(InputMethod::Vni);assert!(e.buffer().is_empty());
    let mut e=Engine::new();e.process_char('a');e.process_char('s');e.on_backspace();assert!(e.buffer().is_empty());
    println!("9 core/engine smoke assertions passed");
}
''',encoding='utf-8')
    binary=work/'engine_probe'
    run(['rustc','+stable','--edition=2024',str(harness),'--extern','funput_core='+str(work/'libfunput_core.rlib'),'--extern','funput_engine='+str(work/'libfunput_engine.rlib'),'-L','dependency='+str(work),'-o',str(binary)])
    results['rust_core_engine_smoke']={'toolchain':'installed stable 1.99.0; upstream pin 1.98.1 not installed','result':run([str(binary)]).strip(),'cargo_workspace_tests':False}

    surface_source=(root/'platforms/macos/Funput/IME/Composition/CompositionSurface.swift').read_text()
    start=surface_source.index('protocol CompositionSurface')
    begin=surface_source.index('{',start);position=begin+1;depth=1
    while depth:
        depth+=(surface_source[position]=='{')-(surface_source[position]=='}');position+=1
    surface_protocol=surface_source[start:position]
    fake='''
final class FixtureSurface: CompositionSurface {
    var document=""
    var caret=0
    var reported: NSRange?=nil
    var supportsDocumentAccess: Bool { true }
    var clientID: String? { "org.fixture.editor" }
    func selection() -> NSRange? { reported ?? NSRange(location:caret,length:0) }
    func text(in range:NSRange) -> String? {
        let s=document as NSString
        guard range.location>=0,range.length>=0,NSMaxRange(range)<=s.length else{return nil}
        return s.substring(with:range)
    }
    func insert(_ text:String,replacing range:NSRange?) {
        let r=range ?? NSRange(location:caret,length:0)
        document=(document as NSString).replacingCharacters(in:r,with:text)
        caret=r.location+text.utf16.count
    }
    func mark(_ text:String,replacing range:NSRange?) {insert(text,replacing:range)}
}
let suffix="KEEP_THIS_SUFFIX"
func probe(_ badSelection:Bool) -> [String:Any] {
    let s=FixtureSurface();var p=DirectPresenter()
    _=p.show("go",on:s)
    s.document += suffix
    if badSelection {s.reported=NSRange(location:2,length:suffix.utf16.count)}
    let result=p.show("gơ",on:s)
    return ["reported_selection_is_false":badSelection,"result":String(describing:result),"suffix_preserved":s.document.hasSuffix(suffix),"document_after":s.document]
}
let control=probe(false),risk=probe(true)
precondition(control["suffix_preserved"] as? Bool == true)
precondition(risk["suffix_preserved"] as? Bool == false)
let json=try JSONSerialization.data(withJSONObject:["control":control,"false_selection":risk],options:[.sortedKeys])
print(String(data:json,encoding:.utf8)!)
'''
    swift=work/'main.swift'
    swift.write_text('import Foundation\n'+surface_protocol+'\n'+(root/'platforms/macos/Funput/IME/Composition/CompositionPresenter.swift').read_text()+'\n'+(root/'platforms/macos/Funput/IME/Composition/DirectPresenter.swift').read_text()+'\n'+fake,encoding='utf-8')
    binary=work/'presenter_probe'
    run(['/usr/bin/xcrun','swiftc','-module-cache-path',str(work/'swift_module_cache'),str(swift),'-o',str(binary)])
    results['macos_direct_presenter_false_selection']=json.loads(run([str(binary)]))

files=['LICENSE','Cargo.toml','Cargo.lock','rust-toolchain.toml','deny.toml','platforms/macos/Funput/Info.plist','platforms/macos/Funput/IME/Composition/DirectPresenter.swift','platforms/macos/Funput/IME/Composition/ClientTrust.swift','platforms/windows/src/background/inject/events.rs','platforms/windows/src/background/hook/keyboard/keydown.rs','platforms/windows/src/shared/update/mod.rs','crates/funput-suggestions/data/lexicon/NOTICE.md']
evidence={'date':'2026-10-27','upstream_head':'2940d6aa04a9d90ffe24327732d843784be1adaf','scope':'Audit only. Compiled actual Rust core/engine and Swift presenter/protocol source with synthetic surface; no real keyboard injection, install, input-source switch or user-document modification. Not full Cargo/XCTest/native GUI coverage.','results':results,'source_sha256':{file:hashlib.sha256((root/file).read_bytes()).hexdigest() for file in files}}
output=Path(__file__).resolve().with_name('audit_evidence_probe.json')
output.write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(results,ensure_ascii=False))
