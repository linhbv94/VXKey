import type { Envelope, MacroEntry } from '../types';
export type MacroPage = { items: MacroEntry[]; revision: string };
/** Whole-list replacement is safe only after every page of one revision was read. */
export async function loadAllMacros(
  revision: string,
  read: (offset: number, limit: number) => Promise<Envelope<MacroPage>>,
): Promise<MacroEntry[]> {
  const entries: MacroEntry[] = [];
  const ids = new Set<string>();
  for (let offset = 0; offset <= 1000; offset += 50) {
    const result = await read(offset, 50);
    if (!result.ok || !result.data) throw new Error(result.error?.code || 'unavailable');
    if (result.data.revision !== revision) throw new Error('revision_conflict');
    const page = result.data.items;
    if (page.length > 50 || entries.length + page.length > 1000) throw new Error('invalid_response');
    for (const entry of page) {
      if (ids.has(entry.id)) throw new Error('invalid_response');
      ids.add(entry.id);
      entries.push(entry);
    }
    if (page.length < 50) return entries;
  }
  throw new Error('quota_exceeded');
}
