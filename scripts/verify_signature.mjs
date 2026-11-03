import assert from 'node:assert/strict';
import { createHash, createPublicKey, verify } from 'node:crypto';

// Minisign format used by Tauri; no secret key is needed for verification.
// Reference: https://github.com/jedisct1/rust-minisign-verify
export function verifySignature(bytes, encodedPublicKey, encodedSignature) {
  const publicLines = Buffer.from(encodedPublicKey, 'base64').toString('utf8').trim().split(/\r?\n/);
  const signatureLines = Buffer.from(encodedSignature, 'base64').toString('utf8').trim().split(/\r?\n/);
  assert.equal(publicLines.length, 2, 'Invalid Tauri public key');
  assert.equal(signatureLines.length, 4, 'Invalid Tauri signature');
  const publicBytes = Buffer.from(publicLines[1], 'base64');
  const signatureBytes = Buffer.from(signatureLines[1], 'base64');
  const globalSignature = Buffer.from(signatureLines[3], 'base64');
  assert.equal(publicBytes.length, 42);
  assert.equal(signatureBytes.length, 74);
  assert.equal(globalSignature.length, 64);
  assert.ok(['Ed', 'ED'].includes(publicBytes.subarray(0, 2).toString()));
  assert.ok(publicBytes.subarray(2, 10).equals(signatureBytes.subarray(2, 10)), 'Signing key differs from the public key in the app');
  const algorithm = signatureBytes.subarray(0, 2).toString();
  assert.ok(['Ed', 'ED'].includes(algorithm), 'Unsupported signature algorithm');
  assert.ok(signatureLines[2].startsWith('trusted comment: '));
  const key = createPublicKey({
    key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), publicBytes.subarray(10)]),
    format: 'der', type: 'spki',
  });
  const data = algorithm === 'ED' ? createHash('blake2b512').update(bytes).digest() : bytes;
  const signature = signatureBytes.subarray(10);
  assert.ok(verify(null, data, key, signature), 'Updater package signature is invalid');
  const trustedComment = Buffer.from(signatureLines[2].slice('trusted comment: '.length));
  assert.ok(verify(null, Buffer.concat([signature, trustedComment]), key, globalSignature), 'Signature trusted comment is invalid');
}
