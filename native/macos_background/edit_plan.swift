import Foundation

/// Pure policy: deletion counts are engine Unicode scalars, not UTF-16 offsets.
struct EditPlan {
    let backspaces: Int
    let output: String
    let resultingText: String
    /// Reconcile observed text with the composer's retained result after an AX delay.
    static func reconcile(owned: String, target: String) -> EditPlan? {
        let actual=Array(owned.unicodeScalars), desired=Array(target.unicodeScalars)
        var common=0
        while common<min(actual.count,desired.count), actual[common]==desired[common] { common+=1 }
        let suffix=String(String.UnicodeScalarView(desired.dropFirst(common)))
        return make(owned:owned,backspaces:actual.count-common,output:suffix)
    }
    static func unicodeChunks(_ text: String) -> [[UInt16]] {
        var chunks: [[UInt16]] = []; var current: [UInt16] = []
        for scalar in text.unicodeScalars {
            let units = Array(String(scalar).utf16)
            if current.count + units.count > 20 { chunks.append(current); current = [] }
            current.append(contentsOf: units)
        }
        if !current.isEmpty { chunks.append(current) }
        return chunks
    }
    static func make(owned: String, backspaces: Int, output: String) -> EditPlan? {
        guard backspaces >= 0, backspaces <= owned.unicodeScalars.count,
              owned.count == owned.unicodeScalars.count,
              owned.unicodeScalars.allSatisfy({ $0.properties.canonicalCombiningClass == .notReordered && $0.value <= 0xffff }),
              output.utf16.count <= 8192, backspaces <= 128,
              !output.unicodeScalars.contains(where: { $0.value == 0 }) else { return nil }
        let prefix = String(String.UnicodeScalarView(owned.unicodeScalars.dropLast(backspaces)))
        return EditPlan(backspaces: backspaces, output: output, resultingText: prefix + output)
    }
}
