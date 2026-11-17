// Independent protocol assembly and OpenSSL ECDSA. No Auth/Platform code loaded.
import { createHash, createECDH, createPrivateKey, createPublicKey, sign, verify } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const cat = (...parts) => Buffer.concat(parts), text = s => Buffer.from(s), rep = (n, len=32) => Buffer.alloc(len,n);
const u32 = n => { const b=Buffer.alloc(4); b.writeUInt32BE(n); return b; };
const u64 = n => { const b=Buffer.alloc(8); b.writeBigUInt64BE(BigInt(n)); return b; };
const bytes = b => cat(u32(b.length),b), sha = b => createHash('sha256').update(b).digest();
const certified = process.argv.includes('--certified');
const installed = process.argv.includes('--installed');
assert(!installed || certified);
const time = n => installed ? 2_000_000_000_000_000_000n + BigInt(n) * 1_000_000_000n : n;
const duration = n => installed ? BigInt(n) * 1_000_000_000n : n;
const otherIssuer = process.argv.includes('--other-issuer');
assert(!otherIssuer || certified, '--other-issuer requires --certified');
const seedHash = certified ? sha(text('auth-qualification-issuer')) : rep(8);
const path = cat(text('CANIC_ROOT_DELEGATION_CHAIN_KEY_DERIVATION_PATH_V1'),u32(2),bytes(text('canic')),bytes(text('delegation')));
const pathHash = sha(path), root = bytes(rep(1,10)), issuer = bytes(rep(otherIssuer ? 3 : 2,10));
const binding = sha(cat(text('canic-issuer-proof-binding-v1'),issuer,Buffer.from([1,1]),seedHash));
const authorization = cat(Buffer.from([1]),binding,Buffer.from([1]),seedHash);
const grants = cat(u32(1),bytes(text('app')),u32(2),bytes(text('read')),bytes(text('write')));
const audience = cat(Buffer.from([1]),rep(3),rep(4));
const cert = cat(text('CANIC-AUTH\0'),Buffer.from([1]),root,issuer,authorization,u64(time(100)),u64(time(100)),u64(time(200)),u64(duration(60)),audience,grants);
const leaf = cat(root,issuer,u64(2),authorization,u64(duration(60)),audience,grants,u64(time(100)),u64(time(200)),u64(3),rep(9));
const leafHash = sha(cat(Buffer.from([0]),text('CANIC_ROOT_DELEGATION_CHAIN_KEY_ISSUER_LEAF_V1'),bytes(leaf)));
// First a left sibling, then a right sibling: order and direction both matter.
const treeRoot = sha(cat(Buffer.from([1]),sha(cat(Buffer.from([1]),rep(11),leafHash)),rep(12)));
const header = cat(Buffer.from([0,1]),root,rep(10),u64(2),u64(3),rep(9),treeRoot,u64(time(100)),u64(time(200)),Buffer.from([1]),bytes(text('test_key_1')),pathHash,u64(4));
const preimage = cat(text('CANIC_ROOT_DELEGATION_CHAIN_KEY_BATCH_V1'),bytes(header));
const testScalar = Buffer.alloc(32); testScalar[31]=1; // Public fixture key; never runtime custody.
const ecdh=createECDH('secp256k1'); ecdh.setPrivateKey(testScalar);
const point=ecdh.getPublicKey(null,'uncompressed');
const privateKey=createPrivateKey({format:'jwk', key:{kty:'EC',crv:'secp256k1',d:testScalar.toString('base64url'),x:point.subarray(1,33).toString('base64url'),y:point.subarray(33).toString('base64url')}});
const publicKey=createPublicKey(privateKey);
const order=BigInt('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141');
const scalar = b => BigInt(`0x${b.toString('hex')}`), encoded = n => Buffer.from(n.toString(16).padStart(64,'0'),'hex');
const target=new URL(installed ? './fixtures/certified-installed.tsv' : otherIssuer ? './fixtures/certified-other-issuer.tsv' : certified ? './fixtures/certified.tsv' : './fixtures/root-delegation.tsv',import.meta.url);
let signature;
if(process.argv.includes('--generate')) {
    signature=sign('sha256',preimage,{key:privateKey,dsaEncoding:'ieee-p1363'});
    const s=scalar(signature.subarray(32));
    if(s>order/2n) signature=cat(signature.subarray(0,32),encoded(order-s));
} else {
    const records=new Map(readFileSync(target,'utf8').trim().split('\n').map(l=>l.split('\t')));
    signature=Buffer.from(records.get('signature'),'hex');
}
const high=cat(signature.subarray(0,32),encoded(order-scalar(signature.subarray(32))));
assert(scalar(signature.subarray(32))<=order/2n);
// OpenSSL accepts both equivalent signatures; Auth must refuse the high-S form.
for(const s of [signature,high]) assert(verify('sha256',preimage,{key:publicKey,dsaEncoding:'ieee-p1363'},s));
const records={binding,certificate_hash:sha(cert),path_hash:pathHash,leaf_hash:leafHash,tree_root:treeRoot,header_hash:sha(preimage),public_key:ecdh.getPublicKey(null,'compressed'),public_key_uncompressed:point,signature,high_signature:high};
if(certified) {
    const browser=Buffer.from('03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8','hex');
    const caller=createHash('sha224').update(cat(Buffer.from('302a300506032b6570032100','hex'),browser)).digest();
    const claims=cat(text('IC-AUTH-PORTABLE-CREDENTIAL\0'),Buffer.from([0,2,0]),rep(3),rep(4),issuer,sha(cert),browser,bytes(rep(5,10)),Buffer.from([1]),u64(7),u64(time(100)),u64(time(190)),u64(time(140)),u64(time(180)),rep(6,16),grants);
    const edKey=createPrivateKey({format:'der',type:'pkcs8',key:cat(Buffer.from('302e020100300506032b657004220420','hex'),Buffer.from(Array.from({length:32},(_,i)=>i)))});
    const edPublic=createPublicKey(edKey).export({format:'der',type:'spki'}).subarray(-32);
    const edClaims=cat(text('IC-AUTH-PORTABLE-CREDENTIAL\0'),Buffer.from([0,2,1]),rep(3),rep(4),issuer,sha(edPublic),browser,bytes(rep(5,10)),Buffer.from([1]),u64(7),u64(time(100)),u64(time(190)),u64(time(140)),u64(time(180)),rep(6,16),grants);
    const edSignature=sign(null,sha(edClaims),edKey);
    assert(verify(null,sha(edClaims),createPublicKey(edKey),edSignature));
    Object.assign(records,{seed_hash:seedHash,claims,claims_digest:sha(claims),caller:cat(caller,Buffer.from([2])),ed_claims:edClaims,ed_public_key:edPublic,ed_key_binding:sha(edPublic),ed_signature:edSignature});
}
const output=Object.entries(records).map(([k,v])=>`${k}\t${v.toString('hex')}\n`).join('');
if(process.argv.includes('--generate')) writeFileSync(target,output);
else assert.equal(readFileSync(target,'utf8'),output);
console.log('Independent root commitments and low/high-S OpenSSL controls pass.');
