import { test } from 'node:test';
import assert from 'node:assert/strict';
import { loadAllMacros } from '../src/services/macro_loader.ts';
const all = Array.from({length: 123}, (_, i) => ({id: `${i}`, trigger: `k${i}`, replacement: `Text ${i}`}));
test('loading beyond first page retains all macros for whole-list save', async () => {
  const offsets: number[] = [];
  const result = await loadAllMacros('5', async (offset, limit) => {
    offsets.push(offset);
    return {ok:true, request_id:'fixture', error:null, data:{items:all.slice(offset, offset+limit),revision:'5'}};
  });
  assert.deepEqual(result, all);
  assert.deepEqual(offsets, [0,50,100]);
});
test('failed page or changed revision never yields partial dictionary', async () => {
  for (const changed of [true, false]) {
    await assert.rejects(loadAllMacros('5', async (offset) => offset === 0
      ? {ok:true,request_id:'fixture',error:null,data:{items:all.slice(0,50),revision:'5'}}
      : changed ? {ok:true,request_id:'fixture',error:null,data:{items:all.slice(50),revision:'6'}}
        : {ok:false,request_id:'fixture',data:null,error:{code:'unavailable',message_key:'errors.unavailable',retryable:true}}));
  }
});
