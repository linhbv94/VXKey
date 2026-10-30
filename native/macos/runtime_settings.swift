// VXKey native cache adapter. Upstream composer/controller remain Rust C-ABI based.
import AppKit
import InputMethodKit
import Foundation
import Carbon

struct TextShortcut {let trigger:String;let expansion:String;var isComplete:Bool{!trigger.isEmpty && !expansion.isEmpty}}
struct KeyCombo {
    let kind:String;let modifiers:Set<String>;let keyCode:UInt16?
    static let defaultToggle=KeyCombo(kind:"modifier_chord",modifiers:["control","alt"],keyCode:nil)
    static func decode(_ value:Any?)->KeyCombo?{
        guard let v=value as? [String:Any],let kind=v["kind"] as? String,let raw=v["modifiers"] as? [String],!raw.isEmpty else{return nil}
        let keys:[String:UInt16]=["KeyV":9,"KeyZ":6,"Space":49,"Backslash":42,"KeyA":0,"KeyC":8,"KeyX":7,"KeyS":1]
        if kind=="modifier_chord",raw.count>=2{return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:nil)}
        guard kind=="key_chord",let key=v["key_code"] as? String,let code=keys[key]else{return nil}
        return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:code)
    }
    static func families(_ f:NSEvent.ModifierFlags)->Set<String>{var result=Set<String>();if f.contains(.control){result.insert("control")};if f.contains(.option){result.insert("alt")};if f.contains(.shift){result.insert("shift")};if f.contains(.command){result.insert("meta")};return result}
    func matches(_ e:NSEvent)->Bool{kind=="key_chord" && keyCode==e.keyCode && Self.families(e.modifierFlags)==modifiers}
}
final class NativeModifierGesture {
    private var down:Set<UInt16>=[];private var armed=false;private var cancelled=false
    private let names:[UInt16:String]=[59:"control",62:"control",58:"alt",61:"alt",56:"shift",60:"shift",55:"meta",54:"meta"]
    func reset(){down=[];armed=false;cancelled=false}
    func nonModifier(){if !down.isEmpty{cancelled=true;armed=false}}
    func handle(_ event:NSEvent,combo:KeyCombo)->Bool{
        guard combo.kind=="modifier_chord",let family=names[event.keyCode]else{return false}
        let pressed=CGEventSource.keyState(.combinedSessionState,key:CGKeyCode(event.keyCode));let hadDown = !down.isEmpty
        if pressed{down.insert(event.keyCode)}else{down.remove(event.keyCode)}
        if !combo.modifiers.contains(family){cancelled=true;armed=false}
        let families=Set(down.compactMap{names[$0]})
        if families==combo.modifiers && !cancelled {armed=true}
        if hadDown && down.isEmpty{let fire=armed && !cancelled;reset();return fire}
        return false
    }
}

final class AppSettings {
    static let shared=AppSettings()
    var inputMethod:InputMethod = .telex;var toneStyle:ToneStyle = .modern
    var vietnameseEnabled=true;var smartEnglishRestore=true;var eagerRestore=true
    var spellCheckEnabled=false;var autoCapitalizeEnabled=false;var extraOnsets:ExtraOnsetLetters=[]
    var retoneAfterBackspace=true;var directComposition=false
    var shortcutsEnabled=true;var shortcutSmartCase=true;var shortcutsInEnglish=true
    var shortcuts:[TextShortcut]=[];var shortcutsRevision=0
    var toggleShortcut=KeyCombo.defaultToggle;var flipShortcut:KeyCombo?
    var suspended=false;private var policies:[String:[String:Bool]]=[:]
    private var memory:[String:Bool]=[:];private var localMemory:[String:Bool]=[:];private var remember=true
    private var revision="";private var timer:DispatchSourceTimer?
    private let loader=DispatchQueue(label:"com.vxkey.native.config",qos:.utility)
    var dataRoot:URL {
        if let path=ProcessInfo.processInfo.environment["VXKEY_DATA_DIR"]{return URL(fileURLWithPath:path)}
        return FileManager.default.urls(for:.applicationSupportDirectory,in:.userDomainMask)[0].appendingPathComponent("VXKey",isDirectory:true)
    }
    func start(){
        let source=DispatchSource.makeTimerSource(queue:loader);source.schedule(deadline:.now(),repeating:.milliseconds(500))
        source.setEventHandler{[weak self] in self?.load()};timer=source;source.resume()
    }
    private func boundedRead(_ url:URL, limit:Int)->Data? {
        guard let file=try? FileHandle(forReadingFrom:url)else{return nil}
        defer{try? file.close()}
        guard let data=try? file.read(upToCount:limit+1),data.count<=limit else{return nil}
        return data
    }
    private func load(){
        let url=dataRoot.appendingPathComponent("config.json")
        let fallback=Bundle.main.url(forResource:"default_config",withExtension:"json")
        guard let data=boundedRead(FileManager.default.fileExists(atPath:url.path) ? url : (fallback ?? url),limit:1024*1024),data.count<=1024*1024,
              let doc=(try? JSONSerialization.jsonObject(with:data)) as? [String:Any],doc["schema_version"] as? Int==1,
              let rev=doc["revision"] as? String,UInt64(rev) != nil,let typing=doc["typing"] as? [String:Any],
              let method=typing["input_method"] as? String,["telex","telex_advanced","vni"].contains(method)
        else {DispatchQueue.main.async{self.suspended=true};return}
        let marker=dataRoot.appendingPathComponent("recovery_state.json")
        let markerExists=FileManager.default.fileExists(atPath:marker.path)
        let safe:Bool
        if markerExists,let bytes=boundedRead(marker,limit:4096),let state=(try? JSONSerialization.jsonObject(with:bytes)) as? [String:Any],let value=state["suspended"] as? Bool{safe=value}else{safe=markerExists}
        DispatchQueue.main.async {
            self.suspended=safe
            guard rev != self.revision else{return};self.revision=rev
            self.inputMethod=InputMethod(rawValue:method) ?? .telex
            self.toneStyle=(typing["tone_style"] as? String)=="traditional" ? .traditional:.modern
            self.vietnameseEnabled=(typing["language_mode"] as? String) != "en"
            self.smartEnglishRestore=typing["smart_restore"] as? Bool ?? true;self.eagerRestore=typing["eager_restore"] as? Bool ?? true
            self.spellCheckEnabled=typing["spell_check"] as? Bool ?? false;self.autoCapitalizeEnabled=typing["auto_capitalize"] as? Bool ?? false
            self.retoneAfterBackspace=typing["retone"] as? Bool ?? true
            self.extraOnsets=ExtraOnsetLetters(configValue:((typing["extra_onsets"] as? [String]) ?? []).joined())
            // Candidate safety profile: retain direct implementation, keep preedit until native matrix accepted.
            self.directComposition=false
            let apps=doc["apps"] as? [String:Any] ?? [:];self.remember=apps["remember_per_app"] as? Bool ?? true
            self.policies=apps["policies"] as? [String:[String:Bool]] ?? [:]
            self.memory=(apps["language_memory"] as? [String:String] ?? [:]).mapValues{$0=="vi"}
            let macros=doc["macros"] as? [String:Any] ?? [:];self.shortcutsEnabled=macros["enabled"] as? Bool ?? true
            self.shortcutsInEnglish=macros["in_en"] as? Bool ?? true;self.shortcutSmartCase=macros["smart_case"] as? Bool ?? true
            self.shortcuts=((macros["entries"] as? [[String:Any]]) ?? []).prefix(1000).compactMap{entry in guard let trigger=entry["trigger"] as? String,let replacement=entry["replacement"] as? String,trigger.count<=32,replacement.utf8.count<=8192 else{return nil};return TextShortcut(trigger:trigger,expansion:replacement)};self.shortcutsRevision+=1
            let keys=doc["shortcuts"] as? [String:Any] ?? [:];if let toggle=KeyCombo.decode(keys["toggle"]){self.toggleShortcut=toggle}else{self.suspended=true};self.flipShortcut=KeyCombo.decode(keys["flip"])
        }
    }
    func shouldBypass(_ client:IMKTextInput)->Bool {
        let id=client.bundleIdentifier() ?? NSWorkspace.shared.frontmostApplication?.bundleIdentifier
        return suspended || IsSecureEventInputEnabled() || (id?.hasPrefix("com.vxkey.") ?? false) || (id.flatMap{policies[$0]?["input_bypass"]} ?? false)
    }
    func resolveVietnamese(for id:String?)->Bool?{guard let id,!id.hasPrefix("com.vxkey."),remember else{return nil};return localMemory[id] ?? memory[id]}
    func pinVietnamese(_ value:Bool,to id:String?){vietnameseEnabled=value;if let id,!id.hasPrefix("com.vxkey."){localMemory[id]=value}}
}
