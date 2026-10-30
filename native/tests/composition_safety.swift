import Foundation

protocol CompositionSurface {
    func insert(_ text: String, replacing range: NSRange?)
    func mark(_ text: String, replacing range: NSRange?)
    func selection() -> NSRange?
    func text(in range: NSRange) -> String?
    var supportsDocumentAccess: Bool { get }
    var clientID: String? { get }
}
final class Fake: CompositionSurface {
    var doc="";var caret=0;var claimedLength=0;var writes=0
    var supportsDocumentAccess:Bool{true};var clientID:String?{"fixture.editor"}
    func insert(_ text:String,replacing range:NSRange?){let r=range ?? NSRange(location:caret,length:0);doc=(doc as NSString).replacingCharacters(in:r,with:text);caret=r.location+text.utf16.count;writes+=1}
    func mark(_ text:String,replacing range:NSRange?){insert(text,replacing:range)}
    func selection()->NSRange?{NSRange(location:caret,length:claimedLength)}
    func text(in range:NSRange)->String?{guard range.location>=0,NSMaxRange(range)<=doc.utf16.count else{return nil};return (doc as NSString).substring(with:range)}
}
@main enum SafetyTests {
 static func main(){
  for suffix in ["KEEP_THIS_SUFFIX","🙂 giữ nguyên","e\u{301} suffix"] {
   let s=Fake();var p=DirectPresenter();assert(p.show("go",on:s) == .shown);s.doc="go"+suffix;s.caret=2;s.claimedLength=suffix.utf16.count;let writes=s.writes
   assert(p.show("gơ",on:s) == .dropped);assert(s.doc=="go"+suffix);assert(s.writes==writes)
  }
  let s=Fake();var p=DirectPresenter();_=p.show("go",on:s);s.doc="goKEEP";s.caret=2
  assert(p.show("gơ",on:s) == .rewritten);assert(s.doc=="gơKEEP")
  s.caret=0;assert(p.show("gờ",on:s) == .dropped);assert(s.doc=="gơKEEP")
  let edit=DirectEdit.Edit(text:"ơ",replacedTail:1);assert(DirectEdit.rewriteRange(edit,selection:NSRange(location:2,length:100))==NSRange(location:1,length:1))
  print("PASS: actual DirectPresenter false-selection suffix, UTF-16, stale caret, collapsed rewrite and range ownership fixtures")
 }
}
