# VXKey - Argon read-only review packet

User explicitly requests Gemini 4 Argon top-tier REVIEW ONLY, not implementation. Latest model list confirms gemini-4-argon-high. This is initial review; automated correction rounds used0/max3; consultation calls0 and no agent/CLI recursion allowed.

Read global AGENTS.md and workspace .agents/AGENTS.md as applicable. NO edits to source, docs, settings, permissions, OS or apps. NO builds, tests, UI automation, subprocess agents or shell mutations. Read supplied packet; return findings in final answer only, Vietnamese. If more context is essential, read only files under this workspace. Do not write a report file yourself. Codex will save your stdout. Do not offer to implement.

Context: Mac26, SDK27, background keyboard utility with ABC/current source unchanged. Rust Funput engine retained; native Swift hook; Tauri UI. Original IMK route retired by user. MKey paused by user. Autostart OFF. Isolated signed ad-hoc candidate plus private bridge pair pins; repeated builds caused TCC regrant churn, no stable production identity yet.

Native evidence chronology (not assumptions):
1. Session/annotated tap filtered every key because eventTargetUnixProcessID was0. Switched to per-PID tap.
2. Own prepare disabled newly created tap; tap_disabled_by_input was delivered after Resume. Removed explicit disable/reuse, invalidate on Pause. Lifecycle generation guards added.
3. Native manual Telex produced malformed strings e.g. tieesng/vieejt. Fault after seen5/rewrite1/verified0. postToPid batch changed to callback proxy tapPostEvent; still fault edit_verification_timeout.
4. Pure filter probe independent of Funput: physical azx -> azX, while z callback returned nil, x modified Unicode in-place. Shows mutation honored but nil suppression not honored in that per-PID trial.
5. Suppression changed to mutate original event.type=.null and return original, covering consumed keyUp/down, shortcuts, both rewrite paths. Pure filter azx -> aX native confirmed by user.
6. Same null_candidate binary restarted after renaming QA-only driver_probe_mode to driver_probe_mode_disabled (no rebuild/permission change). User manual tieengs Vieetj -> tieng Viet with correct diacritics (reported Vietnamese correctly). Then user typed a longer message: 'ok, [Vietnamese text works] ... nayf ddaay (doan cuoi mat roi)' and says ending lost transformation. Native continuous typing remains unstable; do not call daily-driver ready.
7. CUA targeted input cannot validate foreground: it sent text to background TextEdit while NSWorkspace foreground stayed ChatGPT. User manual physical foreground typing is required. Native snapshots/counters exclude raw keys/text.

Review questions:
- Explain most likely mechanisms for correct initial Telex then dropping out mid-sentence. Prioritize source-supported defects vs hypotheses.
- Is pending AX verification policy (150ms, 80ms freshness, next-key fault) structurally compatible with normal/fast typing? Are reset/lease/config/focus/ownership boundaries coherent?
- Audit per-PID tap callback suppression, event batch ordering, synthetic down/up, nonce/PID skip, keyUp tracking and lifecycle. Identify any safety risk including text corruption or stuck modifiers.
- Is current approach reasonably fixable incrementally or does some boundary require redesign? Do not blindly recommend changing to Input Source contrary to user. Compare alternatives only with concrete tradeoffs and evidence.
- Give <=8 ranked actionable findings with exact file:line, trigger, consequence, confidence, minimum fix direction and a discriminating test. Separate verified defects from hypotheses and OS behavior not proven.
- Finish with 3-5 smallest next steps; no code edits.

## Source snapshots (numbered lines)

### native/macos_background/event_driver.swift
```text
1: import AppKit
2: import Carbon
3: import ApplicationServices
4: import os
5: 
6: final class EventDriver {
7:     let composer = FunputComposer()
8:     // QA-only isolated TextEdit probe. Read once at launch, never in a callback.
9:     private let filterProbe: Bool = {
10:         guard ProcessInfo.processInfo.environment["VXKEY_BRIDGE_DEV_TRUST"] == "1" else { return false }
11:         let url=AppSettings.shared.dataRoot.appendingPathComponent("driver_probe_mode")
12:         return (try? String(contentsOf:url,encoding:.utf8).trimmingCharacters(in:.whitespacesAndNewlines)) == "filter"
13:     }()
14:     // Per-PID native probe on this machine honored in-place mutation but not
15:     // a nil callback return. Neutralize the original event in-place as well.
16:     static func suppress(_ event: CGEvent) -> Unmanaged<CGEvent> {
17:         event.type = .null
18:         return Unmanaged.passUnretained(event)
19:     }
20:     static func probeEvent(_ event: CGEvent) -> Unmanaged<CGEvent>? {
21:         let code=event.getIntegerValueField(.keyboardEventKeycode)
22:         if code == 6 { return suppress(event) } // z: neutralize in-place, no injection
23:         if code == 7 {
24:             let units=Array("X".utf16)
25:             event.keyboardSetUnicodeString(stringLength:units.count,unicodeString:units)
26:         }
27:         return Unmanaged.passUnretained(event)
28:     }
29:     private let context = ContextGuard()
30:     final class TapContext {
31:         weak var driver: EventDriver?
32:         let generation: UInt64
33:         init(_ driver: EventDriver, _ generation: UInt64) { self.driver=driver; self.generation=generation }
34:     }
35:     private var tapGeneration: UInt64 = 0
36:     private var tapContext: TapContext?
37:     private let logger = Logger(subsystem: "com.vxkey.engine", category: "event_driver")
38:     private var tap: CFMachPort?
39:     private var source: CFRunLoopSource?
40:     private var running = false
41:     private var requested = false
42:     private var tapPID: Int32?
43:     private(set) var failed = false
44:     private var owned = ""
45:     private var start = -1
46:     private var epoch: UInt64 = 0
47:     private var pendingAt: TimeInterval?
48:     private var consumed = Set<UInt16>()
49:     private let gesture = NativeModifierGesture()
50:     private let nonce = Int64.random(in: 1...Int64.max)
51:     private var configuredRevision = -1
52:     private var counts: [String:UInt32] = ["seen":0,"bypassed":0,"missing_target":0,"unverified":0,"rewrites":0,"verified":0,"failed":0]
53:     var diagnostics: [String:UInt32] { counts }
54:     private func note(_ key:String) { counts[key] = min(1_000_000,(counts[key] ?? 0)+1) }
55:     var permissionStatus: [String: Bool] { ["accessibility": AXIsProcessTrusted(), "listen_access": CGPreflightListenEventAccess(), "post_access": CGPreflightPostEventAccess(), "attached": attached] }
56:     var attached: Bool { running && tap.map { CGEvent.tapIsEnabled(tap: $0) } == true }
57:     var permissionsReady: Bool { AXIsProcessTrusted() && CGPreflightPostEventAccess() }
58:     func prepare() -> Bool {
59:         guard permissionsReady else { return false }
60:         guard let pid=AppSettings.shared.targetPIDValue else { failed=false; return true }
61:         if tap != nil && tapPID == pid { failed = false; return true }
62:         removeTap()
63:         let mask = [CGEventType.keyDown, .keyUp, .flagsChanged, .leftMouseDown, .rightMouseDown, .otherMouseDown]
64:             .reduce(CGEventMask(0)) { $0 | (CGEventMask(1) << $1.rawValue) }
65:         let callbackContext=TapContext(self,tapGeneration)
66:         guard let tap = CGEvent.tapCreateForPid(pid: pid, place: .headInsertEventTap,
67:             options: .defaultTap, eventsOfInterest: mask, callback: { proxy, type, event, pointer in
68:                 guard let pointer else { return Unmanaged.passUnretained(event) }
69:                 let context=Unmanaged<TapContext>.fromOpaque(pointer).takeUnretainedValue()
70:                 return context.driver?.handle(type,event,generation:context.generation,proxy:proxy) ?? Unmanaged.passUnretained(event)
71:             }, userInfo: Unmanaged.passUnretained(callbackContext).toOpaque()),
72:               let source = CFMachPortCreateRunLoopSource(nil, tap, 0) else { logger.error("tap_prepare_failed"); return false }
73:         self.tapContext=callbackContext
74:         self.tap = tap; self.source = source; self.tapPID=pid
75:         // Newly created taps are enabled. Readiness callbacks pass through
76:         // while running is false. Disabling here queues a delayed notification
77:         // that can otherwise fault the next Resume.
78:         CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes)
79:         failed = false
80:         return true
81:     }
82:     private func removeTap() {
83:         running=false
84:         tapGeneration &+= 1
85:         if let tap { CFMachPortInvalidate(tap) }
86:         if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(),source,.commonModes) }
87:         tap=nil; source=nil; tapPID=nil; tapContext=nil
88:     }
89:     func targetChanged() {
90:         let resume=requested
91:         removeTap()
92:         if resume { setRunning(true) }
93:     }
94:     func setRunning(_ enabled: Bool) {
95:         requested=enabled
96:         if !enabled { if running { reset() }; removeTap(); return }
97:         let allowed=enabled && permissionsReady && !failed
98:         if allowed, tapPID != AppSettings.shared.targetPIDValue || tap == nil {
99:             guard prepare() else { fault(); return }
100:         }
101:         let next=allowed && tap != nil
102:         if next != running { reset(); running=next; if let tap { CGEvent.tapEnable(tap:tap,enable:next) } }
103:     }
104:     func reset() {
105:         if !vxkey_native_engine_faulted() { composer.clear() }
106:         owned = ""; start = -1; pendingAt = nil; epoch &+= 1
107:         if let pid = AppSettings.shared.targetPIDValue { context.expect(pid: pid, text: "", start: -1, epoch: epoch) }
108:     }
109:     func syncSettings() {
110:         guard !vxkey_native_engine_faulted() else { return }
111:         let s = AppSettings.shared
112:         composer.apply(ComposerConfiguration(settings: s))
113:         if configuredRevision != s.shortcutsRevision {
114:             composer.clearShortcuts()
115:             for shortcut in s.shortcuts { composer.addShortcut(trigger: shortcut.trigger, expansion: shortcut.expansion) }
116:             configuredRevision = s.shortcutsRevision
117:         }
118:     }
119:     func verifyPending() {
120:         if running && !attached { fault("tap_not_enabled"); return }
121:         guard let since = pendingAt, let pid = AppSettings.shared.targetPIDValue else { return }
122:         if let s = context.latest(pid: pid, epoch: epoch), s.verified, s.text == owned { pendingAt = nil; note("verified") }
123:         else if ProcessInfo.processInfo.systemUptime - since > 0.15 { fault("edit_verification_timeout") }
124:     }
125:     private func fault(_ reason: String = "edit_or_prepare_failed") {
126:         logger.error("driver_fault reason=\(reason, privacy: .public)")
127:         note("failed"); failed = true; setRunning(false); AppSettings.shared.driverFailed()
128:     }
129:     func batch(_ plan: EditPlan) -> [CGEvent]? {
130:         var events: [CGEvent] = []
131:         // Build every allocation before swallowing the original key. Unicode is
132:         // emitted in bounded UTF-16 chunks; surrogate pairs are never split.
133:         for _ in 0..<plan.backspaces {
134:             for down in [true, false] {
135:                 guard let e = CGEvent(keyboardEventSource: nil, virtualKey: 51, keyDown: down) else { return nil }
136:                 events.append(e)
137:             }
138:         }
139:         for units in EditPlan.unicodeChunks(plan.output) {
140:             for down in [true, false] {
141:                 guard let e = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down) else { return nil }
142:                 e.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
143:                 events.append(e)
144:             }
145:         }
146:         guard events.count <= 1100 else { return nil }
147:         for e in events { e.flags = []; e.setIntegerValueField(.eventSourceUserData, value: nonce) }
148:         return events
149:     }
150:     func handle(_ type: CGEventType, _ event: CGEvent, generation: UInt64, proxy: CGEventTapProxy? = nil) -> Unmanaged<CGEvent>? {
151:         guard generation == tapGeneration else { return Unmanaged.passUnretained(event) }
152:         return handle(type,event,proxy:proxy)
153:     }
154:     func handle(_ type: CGEventType, _ event: CGEvent, proxy: CGEventTapProxy? = nil) -> Unmanaged<CGEvent>? {
155:         let pass = Unmanaged.passUnretained(event)
156:         if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput { if running { fault(type == .tapDisabledByTimeout ? "tap_timeout" : "tap_disabled_by_input") }; return pass }
157:         if event.getIntegerValueField(.eventSourceUserData) == nonce && event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid()) { return pass }
158:         if type == .keyDown { note("seen") }
159:         let code = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
160:         if type == .keyUp, consumed.remove(code) != nil { return Self.suppress(event) }
161:         let settings = AppSettings.shared
162:         guard running, !settings.shouldBypass(), let pid = settings.targetPIDValue else { if type == .keyDown { note("bypassed") }; reset(); gesture.reset(); return pass }
163:         guard tapPID == pid else { if type == .keyDown { note("missing_target") }; reset(); gesture.reset(); return pass }
164:         if type == .flagsChanged {
165:             if let ns = NSEvent(cgEvent: event), let combo = settings.toggleShortcut, gesture.handle(ns, combo: combo) {
166:                 reset(); settings.pinVietnamese(!settings.vietnameseEnabled, to: settings.targetIDValue); syncSettings()
167:             }
168:             return pass
169:         }
170:         guard type == .keyDown else { if type != .keyUp { reset(); gesture.reset() }; return pass }
171:         gesture.nonModifier()
172:         if let ns = NSEvent(cgEvent: event), settings.toggleShortcut?.matches(ns) == true {
173:             if event.getIntegerValueField(.keyboardEventAutorepeat) != 0 { consumed.insert(code); return Self.suppress(event) }
174:             reset(); settings.pinVietnamese(!settings.vietnameseEnabled, to: settings.targetIDValue); syncSettings(); consumed.insert(code); return Self.suppress(event)
175:         }
176:         guard event.flags.intersection([.maskCommand, .maskControl, .maskAlternate]).isEmpty else { reset(); return pass }
177:         if filterProbe {
178:             // Restrict diagnostic behavior to a collapsed, fresh TextEdit field.
179:             // No composer, AX writes, pasteboard, or synthetic posting in probe.
180:             guard settings.targetIDValue == "com.apple.TextEdit",
181:                   let proof=context.latest(pid:pid,epoch:epoch), proof.verified else { return pass }
182:             let result=Self.probeEvent(event)
183:             if code == 6 || code == 7 { note("rewrites") }
184:             if code == 6 { consumed.insert(code) }
185:             return result
186:         }
187:         if pendingAt != nil {
188:             verifyPending()
189:             // Never let another physical key advance the document while keeping
190:             // an earlier ownership proof. That would shift subsequent edits.
191:             if failed { return pass }
192:             if pendingAt != nil { fault("edit_unverified_before_next_key"); return pass }
193:         }
194:         guard let proxy else { reset(); return pass }
195:         if code == 51 {
196:             guard settings.retoneAfterBackspace, !owned.isEmpty,
197:                   let snapshot = context.latest(pid: pid, epoch: epoch), snapshot.verified,
198:                   snapshot.text == owned, snapshot.caret == start + owned.utf16.count,
199:                   EditPlan.make(owned: owned, backspaces: 1, output: "") != nil else { reset(); return pass }
200:             if owned.last == " ", composer.buffer().isEmpty {
201:                 owned.removeLast()
202:                 guard composer.adopt(owned) else { reset(); return pass }
203:             } else {
204:                 let response = composer.backspace()
205:                 if response.action != ACTION_NONE {
206:                     guard response.count < CHARS_CAP,
207:                           let plan = EditPlan.make(owned: owned, backspaces: Int(response.backspace), output: FunputComposer.output(of: response)),
208:                           let events = batch(plan) else { reset(); return pass }
209:                     owned = plan.resultingText; context.expect(pid: pid, text: owned, start: start, epoch: epoch)
210:                     pendingAt = ProcessInfo.processInfo.systemUptime; consumed.insert(code)
211:                     for e in events { e.tapPostEvent(proxy) }; return Self.suppress(event)
212:                 }
213:                 owned.removeLast()
214:             }
215:             if owned.isEmpty { reset() } else { context.expect(pid: pid, text: owned, start: start, epoch: epoch) }
216:             return pass
217:         }
218:         // Navigation/dead keys invalidate ownership. Never adopt an unknown range.
219:         if [36,48,53,115,116,117,119,121,123,124,125,126].contains(Int(code)) { reset(); return pass }
220:         var chars = [UniChar](repeating: 0, count: 8); var count = 0
221:         event.keyboardGetUnicodeString(maxStringLength: chars.count, actualStringLength: &count, unicodeString: &chars)
222:         guard count == 1, let scalar = Unicode.Scalar(UInt32(chars[0])), scalar.value >= 32, scalar.value <= 126 else { reset(); return pass }
223:         guard let snapshot = context.latest(pid: pid, epoch: epoch), snapshot.verified,
224:               snapshot.text == owned, start < 0 || snapshot.caret == start + owned.utf16.count else {
225:             note("unverified")
226:             if let since = pendingAt, ProcessInfo.processInfo.systemUptime - since > 0.15 { fault() }
227:             else if pendingAt == nil { reset() }
228:             return pass
229:         }
230:         if pendingAt != nil { pendingAt = nil }
231:         if owned.last == " ", composer.buffer().isEmpty { owned = ""; start = snapshot.caret }
232:         if start < 0 { start = snapshot.caret }
233:         let result = composer.processText(scalar, source: event.flags.contains(.maskNumericPad) ? .numpad : .standard)
234:         if vxkey_native_engine_faulted() { fault(); return pass }
235:         if result.result.action == ACTION_NONE {
236:             owned.append(Character(String(scalar)))
237:             if owned.utf16.count > 128 || owned.hasSuffix("  ") { reset() }
238:             else { context.expect(pid: pid, text: owned, start: start, epoch: epoch) }
239:             return pass
240:         }
241:         guard let plan = EditPlan.make(owned: owned, backspaces: Int(result.result.backspace), output: result.output),
242:               let events = batch(plan) else { reset(); return pass }
243:         owned = plan.resultingText
244:         context.expect(pid: pid, text: owned, start: start, epoch: epoch)
245:         pendingAt = ProcessInfo.processInfo.systemUptime
246:         consumed.insert(code); note("rewrites")
247:         for e in events { e.tapPostEvent(proxy) }
248:         return Self.suppress(event)
249:     }
250:     deinit {
251:         if let tap { CFMachPortInvalidate(tap) }
252:         if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
253:     }
254: }
```

### native/macos_background/context_guard.swift
```text
1: import AppKit
2: import ApplicationServices
3: 
4: /// Bounded AX probes live off the event-tap run loop. No full value/document reads.
5: final class ContextGuard {
6:     struct Snapshot {
7:         let pid: Int32
8:         let element: AXUIElement
9:         let caret: Int
10:         let start: Int
11:         let text: String
12:         let verified: Bool
13:         let epoch: UInt64
14:         let time: TimeInterval
15:     }
16:     private let lock = NSLock()
17:     private let queue = DispatchQueue(label: "com.vxkey.context")
18:     private var timer: DispatchSourceTimer?
19:     private var pid: Int32 = 0
20:     private var expected = ""
21:     private var start = -1
22:     private var epoch: UInt64 = 0
23:     private var snapshot: Snapshot?
24:     private var element: AXUIElement?
25:     init() {
26:         let timer = DispatchSource.makeTimerSource(queue: queue)
27:         timer.schedule(deadline: .now(), repeating: .milliseconds(10))
28:         timer.setEventHandler { [weak self] in self?.probe() }
29:         self.timer = timer; timer.resume()
30:     }
31:     func expect(pid: Int32, text: String, start: Int, epoch: UInt64) {
32:         // Caller must never wait for an AX RPC. Worker holds this only to copy values.
33:         guard lock.try() else { return }
34:         if self.pid != pid || self.epoch != epoch { element = nil }
35:         self.pid = pid; expected = text; self.start = start; self.epoch = epoch
36:         snapshot = nil; lock.unlock()
37:     }
38:     func latest(pid: Int32, epoch: UInt64) -> Snapshot? {
39:         guard lock.try() else { return nil }
40:         defer { lock.unlock() }
41:         guard let s = snapshot, s.pid == pid, s.epoch == epoch,
42:               ProcessInfo.processInfo.systemUptime - s.time < 0.08 else { return nil }
43:         return s
44:     }
45:     private func probe() {
46:         lock.lock(); let (pid, expected, start, epoch, prior) = (self.pid, self.expected, self.start, self.epoch, element); lock.unlock()
47:         guard pid > 0, AXIsProcessTrusted() else { return }
48:         let system=AXUIElementCreateSystemWide()
49:         AXUIElementSetMessagingTimeout(system,0.04)
50:         var focusedApp:CFTypeRef?
51:         guard AXUIElementCopyAttributeValue(system,kAXFocusedApplicationAttribute as CFString,&focusedApp) == .success,
52:               let focusedApp, CFGetTypeID(focusedApp)==AXUIElementGetTypeID() else { return }
53:         var focusedPID:pid_t=0
54:         guard AXUIElementGetPid(unsafeBitCast(focusedApp,to:AXUIElement.self),&focusedPID) == .success, focusedPID==pid else { return }
55:         let app = AXUIElementCreateApplication(pid)
56:         AXUIElementSetMessagingTimeout(app, 0.04)
57:         var value: CFTypeRef?
58:         guard AXUIElementCopyAttributeValue(app, kAXFocusedUIElementAttribute as CFString, &value) == .success,
59:               let value, CFGetTypeID(value) == AXUIElementGetTypeID() else { return }
60:         let el = unsafeBitCast(value, to: AXUIElement.self)
61:         AXUIElementSetMessagingTimeout(el, 0.04)
62:         var subrole: CFTypeRef?
63:         _ = AXUIElementCopyAttributeValue(el, kAXSubroleAttribute as CFString, &subrole)
64:         guard (subrole as? String) != kAXSecureTextFieldSubrole else { return }
65:         var selected: CFTypeRef?
66:         guard AXUIElementCopyAttributeValue(el, kAXSelectedTextRangeAttribute as CFString, &selected) == .success,
67:               let selected, CFGetTypeID(selected) == AXValueGetTypeID() else { return }
68:         var range = CFRange()
69:         guard AXValueGetValue(unsafeBitCast(selected, to: AXValue.self), .cfRange, &range), range.length == 0, range.location >= 0 else { return }
70:         var verified = expected.isEmpty && start < 0
71:         if !expected.isEmpty, start >= 0, expected.utf16.count <= 8192,
72:            range.location == start + expected.utf16.count,
73:            prior == nil || CFEqual(prior!, el) {
74:             var owned = CFRange(location: start, length: expected.utf16.count)
75:             if let parameter = AXValueCreate(.cfRange, &owned) {
76:                 var found: CFTypeRef?
77:                 verified = AXUIElementCopyParameterizedAttributeValue(el, kAXStringForRangeParameterizedAttribute as CFString, parameter, &found) == .success && (found as? String) == expected
78:             }
79:         }
80:         let result = Snapshot(pid: pid, element: el, caret: range.location, start: start, text: expected, verified: verified, epoch: epoch, time: ProcessInfo.processInfo.systemUptime)
81:         lock.lock()
82:         if self.pid == pid && self.epoch == epoch && self.expected == expected && self.start == start {
83:             snapshot = result
84:             if element == nil { element = el }
85:         }
86:         lock.unlock()
87:     }
88: }
```

### native/macos_background/runtime_settings.swift
```text
1: // Background driver cache. Real Funput composer remains behind the Rust C ABI.
2: import AppKit
3: import Foundation
4: import Carbon
5: 
6: struct TextShortcut {let trigger:String;let expansion:String;var isComplete:Bool{!trigger.isEmpty && !expansion.isEmpty}}
7: struct KeyCombo {
8:     let kind:String;let modifiers:Set<String>;let keyCode:UInt16?
9:     static let defaultToggle=KeyCombo(kind:"modifier_chord",modifiers:["control","alt"],keyCode:nil)
10:     static func decode(_ value:Any?)->KeyCombo?{
11:         guard let v=value as? [String:Any],let kind=v["kind"] as? String,let raw=v["modifiers"] as? [String],!raw.isEmpty else{return nil}
12:         let keys:[String:UInt16]=[
13:             "KeyA":0,"KeyS":1,"KeyD":2,"KeyF":3,"KeyH":4,"KeyG":5,"KeyZ":6,"KeyX":7,"KeyC":8,"KeyV":9,"KeyB":11,
14:             "KeyQ":12,"KeyW":13,"KeyE":14,"KeyR":15,"KeyY":16,"KeyT":17,"Digit1":18,"Digit2":19,"Digit3":20,"Digit4":21,
15:             "Digit6":22,"Digit5":23,"Equal":24,"Digit9":25,"Digit7":26,"Minus":27,"Digit8":28,"Digit0":29,"BracketRight":30,
16:             "KeyO":31,"KeyU":32,"BracketLeft":33,"KeyI":34,"KeyP":35,"Enter":36,"KeyL":37,"KeyJ":38,"Quote":39,"KeyK":40,
17:             "Semicolon":41,"Backslash":42,"Comma":43,"Slash":44,"KeyN":45,"KeyM":46,"Period":47,"Tab":48,"Space":49,"Backquote":50,
18:             "Backspace":51,"Escape":53,"F1":122,"F2":120,"F3":99,"F4":118,"F5":96,"F6":97,"F7":98,"F8":100,"F9":101,
19:             "F10":109,"F11":103,"F12":111,"Home":115,"PageUp":116,"Delete":117,"End":119,"PageDown":121,
20:             "ArrowLeft":123,"ArrowRight":124,"ArrowDown":125,"ArrowUp":126
21:         ]
22:         if kind=="modifier_chord",raw.count>=2{return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:nil)}
23:         guard kind=="key_chord",let key=v["key_code"] as? String,let code=keys[key]else{return nil}
24:         return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:code)
25:     }
26:     static func families(_ f:NSEvent.ModifierFlags)->Set<String>{var result=Set<String>();if f.contains(.control){result.insert("control")};if f.contains(.option){result.insert("alt")};if f.contains(.shift){result.insert("shift")};if f.contains(.command){result.insert("meta")};return result}
27:     func matches(_ e:NSEvent)->Bool{kind=="key_chord" && keyCode==e.keyCode && Self.families(e.modifierFlags)==modifiers}
28: }
29: final class NativeModifierGesture {
30:     private var down:Set<UInt16>=[];private var armed=false;private var cancelled=false
31:     private let names:[UInt16:String]=[59:"control",62:"control",58:"alt",61:"alt",56:"shift",60:"shift",55:"meta",54:"meta"]
32:     func reset(){down=[];armed=false;cancelled=false}
33:     func nonModifier(){if !down.isEmpty{cancelled=true;armed=false}}
34:     func handle(_ event:NSEvent,combo:KeyCombo)->Bool{
35:         guard combo.kind=="modifier_chord" else{return false}
36:         guard let family=names[event.keyCode] else { cancelled=true; armed=false; return false }
37:         let familyDown=KeyCombo.families(event.modifierFlags).contains(family)
38:         let pressed=familyDown && !down.contains(event.keyCode);let hadDown = !down.isEmpty
39:         if pressed{down.insert(event.keyCode)}else{down.remove(event.keyCode)}
40:         if !combo.modifiers.contains(family){cancelled=true;armed=false}
41:         let families=Set(down.compactMap{names[$0]})
42:         if families==combo.modifiers && !cancelled {armed=true}
43:         if hadDown && down.isEmpty{let fire=armed && !cancelled;reset();return fire}
44:         return false
45:     }
46: }
47: 
48: /// Main-thread immutable-cache owner. All I/O and signature checks live in Rust's
49: /// bridge worker. No timer can declare the event loop healthy if that loop stalls.
50: final class AppSettings {
51:     static let shared = AppSettings()
52:     var inputMethod: InputMethod = .telex
53:     var toneStyle: ToneStyle = .modern
54:     var vietnameseEnabled = true
55:     var smartEnglishRestore = true
56:     var eagerRestore = true
57:     var spellCheckEnabled = false
58:     var autoCapitalizeEnabled = false
59:     var extraOnsets: ExtraOnsetLetters = []
60:     var retoneAfterBackspace = true
61:     var directComposition = false
62:     var shortcutsEnabled = true
63:     var shortcutSmartCase = true
64:     var shortcutsInEnglish = true
65:     var shortcuts: [TextShortcut] = []
66:     var shortcutsRevision = 0
67:     var toggleShortcut: KeyCombo? = .defaultToggle
68:     var flipShortcut: KeyCombo?
69:     private(set) var suspended = true
70:     private var policies: [String: Bool] = [:]
71:     private var memory: [String: Bool] = [:]
72:     private var localMemory: [String: Bool] = [:]
73:     private var remember = true
74:     private var revision = "0"
75:     private var savedMode = "vi"
76:     private var pendingConfig: [String: Any]?
77:     private var pendingMemory: [String: Any]?
78:     private var result: [String: String]?
79:     private let engineSession = UUID().uuidString.lowercased()
80:     private var generation: UInt64 = 0
81:     private var sequence: UInt64 = 0
82:     private var foregroundMatches = false
83:     private var targetID: String?
84:     private var targetPID: Int32?
85:     private var timer: Timer?
86:     private var focusObserver: NSObjectProtocol?
87:     private var connected = false
88:     private var linkFault = false
89:     private var latched = true
90:     private var leaseDeadline: TimeInterval = 0
91:     var driver: EventDriver?
92:     var ready: Bool { driver?.permissionsReady == true }
93:     var targetPIDValue: Int32? { targetPID }
94:     var targetIDValue: String? { targetID }
95:     var composing: Bool { !(driver?.composer.buffer().isEmpty ?? true) }
96:     var dataRoot: URL {
97:         if let path = ProcessInfo.processInfo.environment["VXKEY_DATA_DIR"], path.hasPrefix("/") {
98:             return URL(fileURLWithPath: path)
99:         }
100:         return FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
101:             .appendingPathComponent("VXKey", isDirectory: true)
102:     }
103:     func start() {
104:         guard dataRoot.path.withCString({ vxkey_native_start($0) }) else { return }
105:         focusObserver=NSWorkspace.shared.notificationCenter.addObserver(forName:NSWorkspace.didActivateApplicationNotification,object:nil,queue:.main){[weak self] _ in self?.refreshTarget()}
106:         tick()
107:         let timer = Timer(timeInterval: 0.1, repeats: true) { [weak self] _ in self?.tick() }
108:         self.timer = timer
109:         RunLoop.main.add(timer, forMode: .common)
110:     }
111:     private func target() -> [String: Any]? {
112:         guard let id = targetID, let pid = targetPID else { return nil }
113:         return ["app_id": id, "pid": pid, "engine_session_id": engineSession,
114:                 "focus_generation": String(generation)]
115:     }
116:     func updateExternalTarget(id: String?, pid: Int32?) {
117:         guard let id, let pid, pid > 0, !id.hasPrefix("com.vxkey.") else { if foregroundMatches { driver?.reset() }; foregroundMatches=false; return }
118:         foregroundMatches=true
119:         if targetID != id || targetPID != pid {
120:             generation &+= 1
121:             targetID=id; targetPID=pid
122:             if let mode=resolveVietnamese(for:id) { vietnameseEnabled=mode }
123:             // Publish identity before priming the worker for the first keystroke.
124:             driver?.reset(); driver?.syncSettings(); driver?.targetChanged()
125:         }
126:     }
127:     private func refreshTarget() {
128:         let front=NSWorkspace.shared.frontmostApplication
129:         updateExternalTarget(id:front?.bundleIdentifier,pid:front?.processIdentifier)
130:     }
131:     private func tick() {
132:         precondition(Thread.isMainThread)
133:         driver?.verifyPending()
134:         refreshTarget()
135:         sequence &+= 1
136:         let faulted = vxkey_native_engine_faulted()
137:         let composing = !faulted && self.composing
138:         let active = target() != nil
139:         let faultCode: String? = faulted ? "engine_fault" : (driver?.failed == true ? "driver_fault" : (!ready ? "permission_required" : (linkFault ? "bridge_stale" : nil)))
140:         let status: [String: Any] = ["diagnostics": ProcessInfo.processInfo.environment["VXKEY_BRIDGE_DEV_TRUST"] == "1" ? driver?.diagnostics as Any? ?? NSNull() : NSNull(), "driver": driver?.permissionStatus ?? ["accessibility": false, "listen_access": false, "post_access": false, "attached": false], "engine_session_id": engineSession,
141:             "loop_sequence": String(sequence), "active": active, "target": target() as Any? ?? NSNull(),
142:             "composing": composing, "mode": vietnameseEnabled ? "vi" : "en", "suspended": suspended,
143:             "applied_revision": revision, "result": result as Any? ?? NSNull(),
144:             "memory": pendingMemory as Any? ?? NSNull(),
145:             "fault": faultCode as Any? ?? NSNull()]
146:         guard let bytes = try? JSONSerialization.data(withJSONObject: status),
147:               let text = String(data: bytes, encoding: .utf8),
148:               let pointer = text.withCString({ vxkey_native_tick($0) }) else {
149:             linkFault=true; suspended = true; connected = false; driver?.setRunning(false); return
150:         }
151:         defer { vxkey_native_free(pointer) }
152:         let data = Data(bytes: pointer, count: strlen(pointer))
153:         guard let delivery = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
154:             linkFault=true; suspended = true; connected = false; driver?.setRunning(false); return
155:         }
156:         if delivery["busy"] as? Bool == true {
157:             // A try-lock miss is not a disconnect. Keep only the already-issued
158:             // short lease; sustained contention still expires to safe bypass.
159:             if !suspended && ProcessInfo.processInfo.systemUptime >= leaseDeadline {
160:                 linkFault=true; suspended=true; driver?.setRunning(false)
161:             }
162:             return
163:         }
164:         let freshConnection = delivery["connected"] as? Bool ?? false
165:         if connected && !freshConnection { linkFault = true }
166:         connected = freshConnection
167:         latched = delivery["latched"] as? Bool ?? true
168:         if let config = delivery["config"] as? [String: Any] { pendingConfig = config }
169:         // Safe-boundary apply includes macro table and per-app policies in one tick.
170:         // An ACK is published only after the active engine has consumed that cache.
171:         if !faulted, !composing, let config = pendingConfig {
172:             apply(config); pendingConfig = nil
173:             driver?.syncSettings()
174:         }
175:         suspended = linkFault || !connected || latched || revision == "0" || !ready || driver?.failed == true || vxkey_native_engine_faulted()
176:         leaseDeadline = suspended ? 0 : ProcessInfo.processInfo.systemUptime + 0.5
177:         if let control = delivery["control"] as? [String: Any], let id = control["id"] as? String {
178:             var outcome = "unavailable"
179:             if control["action"] as? String == "request_permissions", control["engine_session_id"] as? String == engineSession {
180:                 let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
181:                 _ = AXIsProcessTrustedWithOptions(options)
182:                 // PostEvent is a separate TCC service. Request only from this
183:                 // explicit action and keep any OS prompt off the input run loop.
184:                 DispatchQueue.global(qos: .userInitiated).async {
185:                     if !CGPreflightPostEventAccess() { _ = CGRequestPostEventAccess() }
186:                 }
187:                 outcome = "applied" // request delivered, not permission granted
188:             } else if control["action"] as? String == "suspend",
189:                control["engine_session_id"] as? String == engineSession,
190:                let requested = control["suspended"] as? Bool {
191:                 // Resume is readiness ACK only; durable latch must clear on host
192:                 // before a following authenticated tick can enable transformations.
193:                 if requested { suspended = true; driver?.setRunning(false); outcome = "applied" }
194:                 else if connected && !composing && revision != "0" && !vxkey_native_engine_faulted(), driver?.prepare() == true { linkFault = false; outcome = "applied" }
195:             } else if control["action"] as? String == "set_language" {
196:                 outcome = "stale_target"
197:                 if active, !suspended, !composing,
198:                    let expected = control["target"] as? [String: Any],
199:                    NSDictionary(dictionary: expected).isEqual(to: target() ?? [:]),
200:                    let mode = control["mode"] as? String, ["vi", "en"].contains(mode) {
201:                     pinVietnamese(mode == "vi", to: targetID)
202:                     driver?.syncSettings(); outcome = "applied"
203:                 }
204:             }
205:             result = ["id": id, "outcome": outcome]
206:         }
207:         driver?.setRunning(!suspended)
208:     }
209:     private func apply(_ doc: [String: Any]) {
210:         // Strict schema/limits/shortcut validation has already run in shared Rust.
211:         guard let rev = doc["revision"] as? String, let typing = doc["typing"] as? [String: Any],
212:               let macros = doc["macros"] as? [String: Any] else { suspended = true; return }
213:         let method = typing["input_method"] as? String ?? "telex"
214:         inputMethod = InputMethod(rawValue: method) ?? .telex
215:         toneStyle = typing["tone_style"] as? String == "traditional" ? .traditional : .modern
216:         let newMode = typing["language_mode"] as? String ?? "vi"
217:         if revision == "0" || savedMode != newMode { vietnameseEnabled = newMode == "vi" }
218:         savedMode = newMode
219:         smartEnglishRestore = typing["smart_restore"] as? Bool ?? true
220:         eagerRestore = typing["eager_restore"] as? Bool ?? true
221:         spellCheckEnabled = typing["spell_check"] as? Bool ?? false
222:         autoCapitalizeEnabled = typing["auto_capitalize"] as? Bool ?? false
223:         retoneAfterBackspace = typing["retone"] as? Bool ?? true
224:         extraOnsets = ExtraOnsetLetters(configValue: (typing["extra_onsets"] as? [String] ?? []).joined())
225:         directComposition = false // legacy storage field; background driver uses verified event edits
226:         remember = doc["remember_per_app"] as? Bool ?? true
227:         policies = doc["input_policies"] as? [String: Bool] ?? [:]
228:         let nextMemory = (doc["language_memory"] as? [String: String] ?? [:]).mapValues { $0 == "vi" }
229:         for key in memory.keys where nextMemory[key] == nil { localMemory.removeValue(forKey: key) }
230:         memory = nextMemory
231:         if let pending = pendingMemory, let id = pending["app_id"] as? String,
232:            let mode = pending["mode"] as? String, memory[id] == (mode == "vi") { pendingMemory = nil }
233:         if !remember { localMemory = [:]; pendingMemory = nil }
234:         shortcutsEnabled = macros["enabled"] as? Bool ?? true
235:         shortcutsInEnglish = macros["in_en"] as? Bool ?? true
236:         shortcutSmartCase = macros["smart_case"] as? Bool ?? true
237:         shortcuts = (macros["entries"] as? [[String: Any]] ?? []).compactMap { entry in
238:             guard let trigger = entry["trigger"] as? String, let replacement = entry["replacement"] as? String else { return nil }
239:             return TextShortcut(trigger: trigger, expansion: replacement)
240:         }
241:         shortcutsRevision += 1
242:         toggleShortcut = KeyCombo.decode(doc["toggle"])
243:         flipShortcut = KeyCombo.decode(doc["flip"])
244:         revision = rev
245:         if let mode = resolveVietnamese(for: targetID) { vietnameseEnabled = mode }
246:     }
247:     func shouldBypass() -> Bool {
248:         return vxkey_native_engine_faulted() || suspended || ProcessInfo.processInfo.systemUptime >= leaseDeadline || IsSecureEventInputEnabled() || targetID == nil ||
249:             !foregroundMatches ||
250:             (targetID?.hasPrefix("com.vxkey.") ?? true) || (targetID.flatMap { policies[$0] } ?? false)
251:     }
252:     func driverFailed() { suspended = true; driver?.setRunning(false) }
253:     func resolveVietnamese(for id: String?) -> Bool? {
254:         guard let id, !id.hasPrefix("com.vxkey."), remember else { return nil }
255:         return localMemory[id] ?? memory[id]
256:     }
257:     func pinVietnamese(_ value: Bool, to id: String?) {
258:         vietnameseEnabled = value
259:         if remember, let id, !id.hasPrefix("com.vxkey.") {
260:             localMemory[id] = value
261:             pendingMemory = ["app_id": id, "mode": value ? "vi" : "en", "base_revision": revision]
262:         }
263:     }
264: }
```

### native/macos_background/edit_plan.swift
```text
1: import Foundation
2: 
3: /// Pure policy: deletion counts are engine Unicode scalars, not UTF-16 offsets.
4: struct EditPlan {
5:     let backspaces: Int
6:     let output: String
7:     let resultingText: String
8:     static func unicodeChunks(_ text: String) -> [[UInt16]] {
9:         var chunks: [[UInt16]] = []; var current: [UInt16] = []
10:         for scalar in text.unicodeScalars {
11:             let units = Array(String(scalar).utf16)
12:             if current.count + units.count > 20 { chunks.append(current); current = [] }
13:             current.append(contentsOf: units)
14:         }
15:         if !current.isEmpty { chunks.append(current) }
16:         return chunks
17:     }
18:     static func make(owned: String, backspaces: Int, output: String) -> EditPlan? {
19:         guard backspaces >= 0, backspaces <= owned.unicodeScalars.count,
20:               owned.count == owned.unicodeScalars.count,
21:               owned.unicodeScalars.allSatisfy({ $0.properties.canonicalCombiningClass == .notReordered && $0.value <= 0xffff }),
22:               output.utf16.count <= 8192, backspaces <= 128,
23:               !output.unicodeScalars.contains(where: { $0.value == 0 }) else { return nil }
24:         let prefix = String(String.UnicodeScalarView(owned.unicodeScalars.dropLast(backspaces)))
25:         return EditPlan(backspaces: backspaces, output: output, resultingText: prefix + output)
26:     }
27: }
```

### native/macos_background/main.swift
```text
1: import AppKit
2: import Darwin
3: 
4: // A per-root lifetime lock prevents duplicate taps, including manual helper starts.
5: let application = NSApplication.shared
6: application.setActivationPolicy(.prohibited)
7: let settings = AppSettings.shared
8: let lockURL = settings.dataRoot.appendingPathComponent("engine.lock")
9: let lockFD = open(lockURL.path, O_CREAT | O_RDWR | O_NOFOLLOW, S_IRUSR | S_IWUSR)
10: guard lockFD >= 0, flock(lockFD, LOCK_EX | LOCK_NB) == 0 else { exit(2) }
11: let driver = EventDriver()
12: settings.driver = driver
13: settings.start()
14: // Parent death cannot leave a detached keyboard utility behind. The tap is owned
15: // by this process and disappears on exit. No unrelated process is signalled.
16: let parent = getppid()
17: let parentTimer = Timer(timeInterval: 0.25, repeats: true) { _ in
18:     if getppid() != parent || parent <= 1 { driver.setRunning(false); application.terminate(nil) }
19: }
20: RunLoop.main.add(parentTimer, forMode: .common)
21: application.run()
22: close(lockFD)
```

### platforms/macos/Funput/IME/FunputComposer+Text.swift
```text
1: import Foundation
2: 
3: extension FunputComposer {
4:     /// Decode a `FunputResult`'s inline `chars` (a C `uint32_t[64]`, imported as a
5:     /// tuple) into a `String`.
6:     static func output(of result: FunputResult) -> String {
7:         var copy = result
8:         let count = Int(result.count)
9:         return withUnsafePointer(to: &copy.chars) { tuplePtr in
10:             tuplePtr.withMemoryRebound(to: UInt32.self, capacity: Int(CHARS_CAP)) { buf in
11:                 var view = String.UnicodeScalarView()
12:                 for i in 0..<count {
13:                     if let s = Unicode.Scalar(buf[i]) { view.append(s) }
14:                 }
15:                 return String(view)
16:             }
17:         }
18:     }
19: 
20:     static func scalars(_ codepoints: [UInt32], _ count: UInt) -> String {
21:         var view = String.UnicodeScalarView()
22:         for i in 0..<Int(count) {
23:             if let s = Unicode.Scalar(codepoints[i]) { view.append(s) }
24:         }
25:         return String(view)
26:     }
27: }
```

### platforms/macos/Funput/IME/FunputComposer.swift
```text
1: import Foundation
2: 
3: /// Safe Swift wrapper around the `funput-ffi` C engine handle.
4: ///
5: /// One instance per `IMKInputController` (input is single-threaded on the main
6: /// run loop). Named `FunputComposer` to avoid colliding with the C `FunputEngine`
7: /// type imported from the bridging header.
8: final class FunputComposer {
9:     private let handle: OpaquePointer
10: 
11:     init() {
12:         handle = funput_engine_new()
13:     }
14: 
15:     deinit {
16:         funput_engine_free(handle)
17:     }
18: 
19:     func setEnabled(_ enabled: Bool) {
20:         funput_set_enabled(handle, enabled)
21:     }
22: 
23:     /// Apply the whole engine configuration (method, tone style, and the feature
24:     /// toggles) in one FFI call — the batch equivalent of the former per-option
25:     /// setters. `enabled` is a separate runtime toggle (see `setEnabled`).
26:     func configure(_ config: FunputConfig) {
27:         funput_configure(handle, config)
28:     }
29: 
30:     /// Arm capitalization for the next word (call on focus so the first letter typed
31:     /// in a field is capitalized). A no-op unless auto-capitalize is on.
32:     func armCapitalization() {
33:         funput_arm_capitalization(handle)
34:     }
35: 
36:     func clear() {
37:         funput_clear(handle)
38:     }
39: 
40:     /// Take the owned preedit once, clearing before a client write can re-enter IMK.
41:     func finishComposition() -> String {
42:         let text = buffer()
43:         clear()
44:         return text
45:     }
46: 
47:     /// Remove every text-expansion shortcut (gõ tắt). Pair with `addShortcut` to
48:     /// replace the whole table when syncing from `AppSettings`.
49:     func clearShortcuts() {
50:         funput_clear_shortcuts(handle)
51:     }
52: 
53:     /// The gõ tắt master switch: off, nothing expands and the table is left loaded, so
54:     /// hosts need not re-push their rows around it. Its own FFI call for the same ABI
55:     /// reason as `setShortcutSmartCase` below.
56:     func setShortcutsEnabled(_ on: Bool) {
57:         funput_set_shortcuts_enabled(handle, on)
58:     }
59: 
60:     func setShortcutsInEnglish(_ on: Bool) {
61:         funput_set_shortcuts_in_english(handle, on)
62:     }
63: 
64:     /// Admit `letters` as initial consonants; `[]` admits none. Its own FFI call.
65:     func setExtraOnsets(_ letters: ExtraOnsetLetters) {
66:         funput_set_extra_onsets(handle, letters.rawValue)
67:     }
68: 
69:     /// Smart-case matching for gõ tắt: on, `tp`/`Tp`/`TP` all find the `tp` entry and
70:     /// the expansion is re-cased to match; off, only the exact trigger expands and the
71:     /// expansion is verbatim. Its own FFI call rather than a `FunputConfig` field —
72:     /// that struct crosses the C ABI by value and this app is built separately from the
73:     /// header, so growing it would mismatch silently. `configure` leaves it alone.
74:     func setShortcutSmartCase(_ on: Bool) {
75:         funput_set_shortcut_smart_case(handle, on)
76:     }
77: 
78:     /// Define a text-expansion shortcut: typing `trigger` then a word boundary injects
79:     /// `expansion` (`vn` → `Việt Nam`). Both strings cross the C ABI as UTF-32.
80:     func addShortcut(trigger: String, expansion: String) {
81:         let t = trigger.unicodeScalars.map(\.value)
82:         let e = expansion.unicodeScalars.map(\.value)
83:         funput_add_shortcut(handle, t, UInt(t.count), e, UInt(e.count))
84:     }
85: 
86:     /// The composed syllable buffer — the text shown as marked (underlined) text.
87:     func buffer() -> String {
88:         var out = [UInt32](repeating: 0, count: Int(CHARS_CAP))
89:         while true {
90:             let count = funput_buffer(handle, &out, UInt(out.count))
91:             if count < out.count { return Self.scalars(out, count) }
92:             out = [UInt32](repeating: 0, count: out.count * 2)
93:         }
94:     }
95: 
96:     /// Physical origin of a key, mirroring the engine's `KeySource`. Raw values
97:     /// match the C `SOURCE_STANDARD` / `SOURCE_NUMPAD` constants in `funput.h`. A
98:     /// numpad digit is kept a literal number instead of a VNI tone/shape modifier.
99:     enum KeySource: UInt32 {
100:         case standard = 0 // SOURCE_STANDARD
101:         case numpad = 1 // SOURCE_NUMPAD
102:     }
103: 
104:     /// Feed one Unicode scalar tagged with its physical `source`; returns the
105:     /// platform instruction. Defaults to the main keyboard, so existing call sites
106:     /// are unchanged.
107:     @discardableResult
108:     func process(_ scalar: Unicode.Scalar, source: KeySource = .standard) -> FunputResult {
109:         funput_process_key(handle, scalar.value, source.rawValue)
110:     }
111: 
112:     /// Drop the last composed character (in-composition Backspace).
113:     @discardableResult
114:     func backspace() -> FunputResult {
115:         funput_backspace(handle)
116:     }
117: 
118:     /// Re-open an already-committed word as the live composition, so the next keystroke
119:     /// edits it (`chào` + ⌫ then `s` gives `cháo`). Returns whether the engine took it:
120:     /// only a Vietnamese syllable is — including one still missing a diacritic (`chuc` →
121:     /// `chúc`, `dien` → `diên`) — which keeps English words and URLs literal, so leave the
122:     /// document alone on `false`.
123:     func adopt(_ word: String) -> Bool {
124:         let scalars = word.unicodeScalars.map(\.value)
125:         return funput_adopt(handle, scalars, UInt(scalars.count))
126:     }
127: 
128:     /// Flip the word being composed between its Vietnamese form and its raw
129:     /// keystrokes (`card` ⇄ `cải`), and back on a second call. Returns the engine
130:     /// result; the caller re-renders the marked text from `buffer()` when its
131:     /// action is not `ACTION_NONE` (macOS shows marked text, so the delete+inject
132:     /// payload itself is unused).
133:     func flipComposing() -> FunputResult {
134:         funput_flip_composing(handle)
135:     }
136: 
137:     /// Capture the complete expansion before the FFI's borrowed UTF-8 output expires.
138:     func processText(_ scalar: Unicode.Scalar, source: KeySource = .standard) -> (result: FunputResult, output: String) {
139:         var output = ""
140:         let result = withUnsafeMutablePointer(to: &output) { pointer in
141:             funput_process_key_text(handle, scalar.value, source.rawValue, { context, bytes, count in
142:                 guard let context, let bytes else { return }
143:                 context.assumingMemoryBound(to: String.self).pointee = String(
144:                     decoding: UnsafeBufferPointer(start: bytes, count: Int(count)), as: UTF8.self)
145:             }, pointer)
146:         }
147:         return (result, output)
148:     }
149: }
```

### platforms/macos/Funput/IME/ComposerConfiguration.swift
```text
1: import Foundation
2: 
3: /// A value snapshot of every preference consumed by the input-method composer.
4: /// Keeping the mapping centralized makes settings synchronization atomic.
5: struct ComposerConfiguration: Equatable {
6:     let inputMethod: InputMethod
7:     let toneStyle: ToneStyle
8:     let enabled: Bool
9:     let smartEnglishRestore: Bool
10:     let eagerRestore: Bool
11:     let spellCheckEnabled: Bool
12:     let autoCapitalizeEnabled: Bool
13:     let extraOnsets: ExtraOnsetLetters
14:     let shortcutsEnabled: Bool
15:     let shortcutSmartCase: Bool
16:     let shortcutsInEnglish: Bool
17: 
18:     init(settings: AppSettings) {
19:         inputMethod = settings.inputMethod
20:         toneStyle = settings.toneStyle
21:         enabled = settings.vietnameseEnabled
22:         smartEnglishRestore = settings.smartEnglishRestore
23:         eagerRestore = settings.eagerRestore
24:         spellCheckEnabled = settings.spellCheckEnabled
25:         autoCapitalizeEnabled = settings.autoCapitalizeEnabled
26:         extraOnsets = settings.extraOnsets
27:         shortcutsEnabled = settings.shortcutsEnabled
28:         shortcutSmartCase = settings.shortcutSmartCase
29:         shortcutsInEnglish = settings.shortcutsInEnglish
30:     }
31: 
32:     init(
33:         inputMethod: InputMethod = .telex,
34:         toneStyle: ToneStyle = .modern,
35:         enabled: Bool = true,
36:         smartEnglishRestore: Bool = true,
37:         eagerRestore: Bool = true,
38:         spellCheckEnabled: Bool = false,
39:         autoCapitalizeEnabled: Bool = false,
40:         extraOnsets: ExtraOnsetLetters = [],
41:         shortcutsEnabled: Bool = true,
42:         shortcutSmartCase: Bool = true,
43:         shortcutsInEnglish: Bool = true
44:     ) {
45:         self.inputMethod = inputMethod
46:         self.toneStyle = toneStyle
47:         self.enabled = enabled
48:         self.smartEnglishRestore = smartEnglishRestore
49:         self.eagerRestore = eagerRestore
50:         self.spellCheckEnabled = spellCheckEnabled
51:         self.autoCapitalizeEnabled = autoCapitalizeEnabled
52:         self.extraOnsets = extraOnsets
53:         self.shortcutsEnabled = shortcutsEnabled
54:         self.shortcutSmartCase = shortcutSmartCase
55:         self.shortcutsInEnglish = shortcutsInEnglish
56:     }
57: }
58: 
59: extension FunputComposer {
60:     func apply(_ configuration: ComposerConfiguration) {
61:         configure(
62:             FunputConfig(
63:                 method: configuration.inputMethod.ffiValue,
64:                 tone_style: UInt8(configuration.toneStyle.rawValue),
65:                 smart_restore: configuration.smartEnglishRestore,
66:                 eager_restore: configuration.eagerRestore,
67:                 spell_check: configuration.spellCheckEnabled,
68:                 auto_capitalize: configuration.autoCapitalizeEnabled
69:             )
70:         )
71:         // None of these ride the by-value `FunputConfig`: `enabled` is a runtime
72:         // toggle, and the gõ tắt switches and extra onsets are kept off the C struct for
73:         // ABI stability.
74:         // Order does not matter — `configure` edits the engine config, never replaces it.
75:         setShortcutsEnabled(configuration.shortcutsEnabled)
76:         setShortcutSmartCase(configuration.shortcutSmartCase)
77:         setShortcutsInEnglish(configuration.shortcutsInEnglish)
78:         setExtraOnsets(configuration.extraOnsets)
79:         setEnabled(configuration.enabled)
80:     }
81: }
```

### docs/spec/09_background_driver.md
```text
1: # 09. Bộ gõ chạy nền — hợp đồng macOS
2: 
3: Version: 0.8.0 — Ngày cập nhật: 2026-11-02
4: Skill: `_spec0_system`, `_spec4_api_data`, `_spec5_qa_acceptance`.
5: Trạng thái: quyết định kiến trúc của user; chưa phải nghiệm thu runtime hoặc specs ready.
6: 
7: ## Quyết định và phạm vi
8: 
9: VXKey là tiện ích chạy nền với một tray và Tauri Settings. Người dùng giữ nguyên ABC hoặc nguồn nhập đang dùng, không thêm/chọn VXKey trong Input Sources. Rust Funput core/engine/C ABI được giữ; thay InputMethodKit bằng Swift/CoreGraphics keyboard driver. Không dùng JavaScript để xử lý phím. Windows giữ hướng hook native, dùng chung engine/config/UI khi triển khai sau.
10: 
11: InputMethodKit là hướng thử đã hủy. Các hướng dẫn cài `VXKeyInput.app` vào Library/Input Methods, TISRegister/Enable/Select và fallback chọn ABC không còn là workflow sản phẩm. Các báo cáo IME trước ngày này là bằng chứng lịch sử, không nghiệm thu driver mới. Không đăng ký nguồn nhập, không thay keyboard layout, không dừng MKey hoặc bộ gõ khác.
12: 
13: ## Cấu trúc và ownership
14: 
15: `VXKey.app` (com.vxkey.desktop) chứa `Contents/Resources/VXKeyEngine.app` (com.vxkey.engine). Companion tự khởi chạy đúng helper được đóng gói; không yêu cầu user mở hai app. Helper giữ lock một instance cho mỗi data root. Companion quản lý config/persistence/clipboard; helper giữ composer và bộ đệm từ trong RAM, không gửi phím/văn bản/AX content qua bridge hoặc log.
16: 
17: Status `driver` chứa accessibility/listen_access/post_access/attached; `active` chỉ external target hợp lệ, `driver_status` mới phản ánh tap. Tauri `vxkey_request_permissions` chỉ Settings; ACK không đồng nghĩa đã cấp quyền. Bridge protocol 2 dùng component `engine`, `engine_session_id`, PID và focus_generation. Peer authentication vẫn dùng Security.framework và identity/pins; IME cũ bị từ chối. Không nới trust để chạy helper mới. Ad-hoc development pins không chứng minh quyền được giữ qua update.
18: 
19: ## State và luồng
20: 
21: Launch → suspended → nhận config hợp lệ → user Resume → kiểm quyền và tạo tap → readiness ACK → host xóa durable latch → tick xác thực mới → chạy. Thiếu quyền: giữ suspended, hiển thị hướng dẫn quyền. Không tự bật lại sau timeout, lỗi engine, mất bridge hoặc restart sau fault. Autostart mặc định OFF.
22: 
23: Active filtering tap yêu cầu Accessibility/post access; listen access được đọc riêng, không tự xin quyền không cần thiết. Driver dùng per-application event tap (CGEventTapCreateForPid) và main run loop; callback chỉ dùng cache, engine và edit plan hữu hạn. Không disk/socket/AX IPC hoặc NSWorkspace query đồng bộ trong callback. Tap đăng ký trực tiếp cho PID external hiện hành và được gắn lại khi focus đổi; không đòi eventTargetUnixProcessID vì field này có thể bằng0 ngay trên annotated tap. Callback chỉ xử lý nếu registration PID khớp cached target, AX worker còn xác minh system-wide focused app. Unknown/mismatch chuyển nguyên phím. Focus cache phải publish PID trước khi prime AX context. Native UI/tray không giữ runtime mutex khi dispatch main-thread API. AX worker đọc focused element/selection và đúng phạm vi từ do VXKey sở hữu, không đọc toàn document hay clipboard. Snapshot cũ, focus đổi, selection khác, secure field, Secure Input, app bị bypass hoặc VXKey UI: chuyển nguyên phím và reset ownership. Bypass đứng trước macro/retone/engine.
24: 
25: Phím có Command/Control/Option thông thường và navigation được chuyển nguyên trạng. Ctrl+Option mặc định là gesture nhả hết modifier, bị hủy nếu có phím khác; custom shortcut vẫn theo config. Vi/En và nhớ app độc lập hard bypass. Foreground target lấy từ NSWorkspace, giữ last external app khi mở tray để loại trừ app đó.
26: 
27: Khi engine yêu cầu rewrite, chỉ xóa phần suffix đã xác minh thuộc từ hiện tại, caret collapsed và snapshot còn hạn. Không suy số Backspace từ UTF-16; chỉ chấp nhận scalar/grapheme tương ứng. Chuẩn bị đủ event batch trước khi nuốt phím gốc. Event tự phát có nonce/PID riêng và gửi đúng PID đích. Không phát modifier tổng hợp. Sau rewrite cần AX readback; không chắc chắn → suspend, không replay mù. Các app không cung cấp AX range đáng tin được pass-through, phải ghi coverage thật.
28: 
29: ## Quyền và phục hồi
30: 
31: Kiểm Accessibility và quyền listen/post theo API macOS. Accessibility và PostEvent phải kiểm/hiển thị riêng: AXIsProcessTrusted không thay CGPreflightPostEventAccess. Nút quyền gọi AXIsProcessTrustedWithOptions và CGRequestPostEventAccess; request PostEvent chạy ngoài main input run loop. Chỉ request bằng hành động rõ ràng của user; quyền trong System Settings do user cấp. Không sửa TCC DB. Agent chỉ thay/bật đúng entry qua Settings UI khi user đã xác nhận rõ đúng candidate và quyền; approval cho binary trước không tự chuyển sang binary mới. Trên máy QA hiện tại macOS liệt kê entry **VXKey**, không phải VXKey Engine; UI/hướng dẫn phải dùng tên app OS thực tế, không yêu cầu user tìm entry không tồn tại. UI phải phân biệt helper connected, permissions và tap attached; không gọi chúng là Input Source active. Pause invalidate/tháo tap, không disable rồi reuse port; process chết khiến tap không còn xử lý. Companion chỉ được quản lý child do chính nó khởi chạy. Không đổi layout như phương án phục hồi.
32: 
33: ## Acceptance contract
34: 
35: - BG01: Given ABC đang chọn, when mở VXKey, then không thêm/chọn input source và Settings/tray hoạt động.
36: - BG02: Given chưa cấp quyền, when Resume, then không nuốt phím, không báo ready; có trạng thái thiếu quyền.
37: - BG03: Given driver đã hoạt động, when gõ `tieengs Vieetj`, then đúng text thực trong TextEdit và app WebView; kiểm bằng phím vật lý/automation từng phím, không paste.
38: - BG04: Given prefix/suffix sentinel, emoji/combining/selection/focus đổi, then không sửa ngoài ownership; chưa chứng minh thì literal pass-through.
39: - BG05: Given app bypass hoặc secure input, then macro/retone/double-space đều không chạy.
40: - BG06: Given tap timeout, bridge/companion/engine mất hoặc sleep/wake, then raw keyboard vẫn dùng được, latch giữ suspended và chỉ explicit Resume mới phục hồi.
41: - BG07: Given Pause/quit/relaunch, then không duplicate hook, không chặn key-up/modifier; không ảnh hưởng MKey/layout.
42: - BG08: Kiểm latency và fast typing; fixture không thay native acceptance. Double-space vẫn Mac OFF/Win ON và native interlock như spec cũ; chưa có edit proof thì không bật capability.
43: 
44: ## Demo và giới hạn
45: 
46: Mở một app → mở tray → đúng tên app, loại trừ ngay → bật lại → thử VI/EN. Trong Settings/System xem connection/permissions/driver; cấp quyền bằng nút/hướng dẫn rõ ràng, Resume rồi thử gõ và Pause. Không hướng dẫn user vào Input Sources. QA native, permission retention, production signing và Windows phải có evidence riêng trước daily-driver/release.
47: 
48: Try-lock busy của bridge chỉ giữ lease cũ còn hạn tối đa0.5s, không giả disconnect hoặc làm reset composer từng tick; contention quá hạn → suspend/latch.
49: 
50: ### Delta native edit stream — 2026-11-02
51: 
52: Batch thay thế phải post qua proxy của callback (`CGEvent.tapPostEvent`), tại cùng điểm trong stream; không gọi postToPid cho rewrite của callback hiện hành. Không giữ proxy để sử dụng ngoài callback. Sau batch vẫn cần AX proof; nếu phím vật lý tiếp theo tới mà batch trước chưa verify, suspend và pass phím đó, không dùng tiếp owned word cũ. Timeout verification phải có reason riêng. Đây là contract source đang sửa; chưa có native acceptance cho stream_candidate.
```

### docs/qa/typing_native_evidence.json
```text
1: {
2:   "date": "2026-11-02",
3:   "status": "null_filter_aX_pass_same_binary_composer_resumed_waiting_typing",
4:   "typing_candidate_artifacts": {
5:     "engine": {
6:       "app": "build/typing_candidate/desktop_candidate/VXKey.app/Contents/Resources/VXKeyEngine.app",
7:       "bundle_id": "com.vxkey.engine",
8:       "cdhash": "9fca848e90ca4ea0fcad19dfd15b0c216625f4fa",
9:       "binary_sha256": "0198f0265fd8f3c4b6475e53840648f4d5ebeb4cc27d457a006447785444bc89"
10:     },
11:     "companion": {
12:       "app": "build/typing_candidate/desktop_candidate/VXKey.app",
13:       "bundle_id": "com.vxkey.desktop",
14:       "cdhash": "78889fedf71efbbfe9701349824da2421c897722",
15:       "binary_sha256": "f2afdd7d887a988d80957e3d2537984cfac1befa48ed53d4bb405f5e985b9f68"
16:     }
17:   },
18:   "permissions": {
19:     "accessibility": true,
20:     "post_event": true,
21:     "input_monitoring": true
22:   },
23:   "native_resume_ack": true,
24:   "native_pause_ack": true,
25:   "ui_responsive_after_resume": true,
26:   "test_app": "TextEdit",
27:   "document": "Untitled 6 (task fixture only)",
28:   "input_method": "telex",
29:   "input": "tieengs Vieetj",
30:   "expected": "tiếng Việt",
31:   "observed": "tieengs Vieetj",
32:   "result": "fail",
33:   "driver_counters_after_test": {
34:     "seen": 16,
35:     "missing_target": 16,
36:     "rewrites": 0,
37:     "unverified": 0,
38:     "failed": 0
39:   },
40:   "fix_prepared": "Per-PID tap registration replaces eventTargetUnixProcessID assumption; focus notification + worker system-wide focus guard; bridge busy retains bounded lease",
41:   "key_candidate_artifacts": {
42:     "engine": {
43:       "app": "build/key_candidate/desktop_candidate/VXKey.app/Contents/Resources/VXKeyEngine.app",
44:       "bundle_id": "com.vxkey.engine",
45:       "cdhash": "958e0cc334e1ce1ec80fa29122b90964c65cb08d",
46:       "binary_sha256": "c7e41b6ae59b4cdcd65b137256a7ff27d3c4144d5a68b7ecf1a89f18fa75fec1"
47:     },
48:     "companion": {
49:       "app": "build/key_candidate/desktop_candidate/VXKey.app",
50:       "bundle_id": "com.vxkey.desktop",
51:       "cdhash": "ee087aac1043682979dbafda82e94e7c2780855b",
52:       "binary_sha256": "f317f02c9cab46f5c355acbb92f33b4e1a61f31a6b0ce3daf264c4b7eb1af33e"
53:     }
54:   },
55:   "key_rust_tests_passed": 154,
56:   "swift_checks_passed": 26,
57:   "tray_lock_tests_passed": 4,
58:   "native_typing_after_per_pid_fix": "FAIL: tap faults after Resume/focus transition before any keyDown; repeated once with TextEdit selected before Resume",
59:   "mkey_paused_confirmed_by_user": true,
60:   "os_grants_automated": true,
61:   "autostart_enabled": false,
62:   "no_input_source_registration": true,
63:   "os_grant_scope": "User explicitly approved replacing/enabling only VXKey for exact key_candidate; done via macOS Settings UI, no other entries modified",
64:   "key_candidate_native": {
65:     "permissions": {
66:       "accessibility": true,
67:       "post_access": true,
68:       "listen_access": true
69:     },
70:     "resume_ack": true,
71:     "attached_initially": true,
72:     "seen": 0,
73:     "failed_first_trial": 1,
74:     "failed_after_second_resume": 2,
75:     "rewrites": 0,
76:     "fault": "driver_fault",
77:     "pause_ack": true,
78:     "root_cause": "not yet established: creation, disable/timeout, or lifecycle; current binary has no typed reason",
79:     "vietnamese_readback": false
80:   },
81:   "tap_candidate_native": {
82:     "permissions": {
83:       "accessibility": true,
84:       "post_access": true,
85:       "listen_access": true
86:     },
87:     "grant_action": "explicit user approval; replaced only VXKey via Settings UI",
88:     "resume_ack": true,
89:     "attached_initially": true,
90:     "input": "tieengs Vieetj",
91:     "observed": "tieengs Vieetj",
92:     "seen": 0,
93:     "rewrites": 0,
94:     "failed": 1,
95:     "fault": "driver_fault",
96:     "unified_log": "2026-11-02 12:11:20.898 VXKeyEngine[97381] driver_fault reason=tap_disabled_by_input",
97:     "pause_ack": true,
98:     "vietnamese_readback": false
99:   },
100:   "next_candidate": "build/null_candidate/desktop_candidate/VXKey.app",
101:   "next_candidate_native_tested": false,
102:   "paused_by_user": false,
103:   "tap_resume_candidate_native": {
104:     "permission_action": "User approved exact binary; replaced only VXKey entry via Settings UI; toggle ON",
105:     "permissions": {
106:       "accessibility": true,
107:       "listen_access": true,
108:       "post_access": true
109:     },
110:     "resume_pause_ack": true,
111:     "fault_after_automated_trial": null,
112:     "seen": 0,
113:     "failed": 0,
114:     "rewrites": 0,
115:     "input": "tieengs Vieetj",
116:     "observed": "tieengs Vieetj",
117:     "test_validity": "Foreground precondition NOT met: OS tap inventory reports helper23532 targets ChatGPT8552, TextEdit510; CUA app click/Raise did not change OS target. Raw result is not valid typing acceptance or engine failure evidence.",
118:     "current_state": "Paused after manual failure",
119:     "manual_typing_pending": false
120:   },
121:   "tap_resume_candidate_manual": {
122:     "user_report": [
123:       "tieêsng Vieejt",
124:       "tieengs Vieejt"
125:     ],
126:     "native_trace": {
127:       "seen": 5,
128:       "rewrites": 1,
129:       "verified": 0,
130:       "failed": 1,
131:       "fault": "driver_fault"
132:     },
133:     "fault_log": "2026-11-02 15:56:14.735 driver_fault reason=edit_or_prepare_failed",
134:     "result": "FAIL: physical typing reaches callback and rewrites incorrectly; not an automation-only issue",
135:     "pause_ack": true,
136:     "hypothesis": "postToPid replacement batch not ordered at tap point; pending physical key pass-through keeps stale ownership; not yet native-verified fix"
137:   },
138:   "stream_candidate_native": {
139:     "permission_action": "explicit user approval; replaced only VXKey entry via Settings UI; ON",
140:     "accessibility": true,
141:     "listen_access": true,
142:     "post_access": true,
143:     "resume_ack": true,
144:     "manual_typing_pending": false,
145:     "runtime_typing_passed": false,
146:     "user_report": [
147:       "tiêesng vieejt",
148:       "tieesng vieejt"
149:     ],
150:     "native_trace": {
151:       "seen": 5,
152:       "rewrites": 1,
153:       "verified": 0,
154:       "failed": 1,
155:       "fault": "driver_fault"
156:     },
157:     "fault_reason": "edit_verification_timeout",
158:     "pause_ack": true
159:   },
160:   "filter_probe_native": {
161:     "accessibility": true,
162:     "listen_access": true,
163:     "post_access": true,
164:     "entry_replaced_by_agent_with_explicit_user_approval": true,
165:     "mode": "filter",
166:     "resume_ack": true,
167:     "user_input_pending": null,
168:     "expected": "aX",
169:     "native_result": "azX",
170:     "interpretation": "In-place x mutation worked; nil did not suppress z in this per-PID trial. No general OS claim.",
171:     "pause_ack": true,
172:     "total_session_counters": {
173:       "seen": 149,
174:       "rewrites": 10,
175:       "failed": 0
176:     },
177:     "counter_scope": "Whole session, not only three keys"
178:   },
179:   "null_probe_native": {
180:     "grant": "Explicit user approval; replaced only VXKey entry via Settings UI; ON",
181:     "accessibility": true,
182:     "listen_access": true,
183:     "post_access": true,
184:     "resume_ack": true,
185:     "mode": "filter",
186:     "expected": "aX",
187:     "result": "aX",
188:     "scope": "User manual TextEdit trial only",
189:     "pause_ack": true,
190:     "whole_session_counters": {
191:       "seen": 25,
192:       "rewrites": 4,
193:       "failed": 0
194:     }
195:   },
196:   "null_composer_native": {
197:     "probe_disabled": "driver_probe_mode renamed driver_probe_mode_disabled after Quit",
198:     "same_binary_hashes_verified": true,
199:     "permission_changed": false,
200:     "resume_ack": true,
201:     "typing_result": "pending"
202:   }
203: }
```
