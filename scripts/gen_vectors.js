// Generates tests/vectors.json by running api-enhanced's util/crypto.js with injected randomness.
// Usage: node scripts/gen_vectors.js <path-to-api-enhanced>/util/crypto.js
// (needs `crypto-js` and `node-forge` resolvable from that file; './zstd' may be stubbed)
const path = require('path')
const fs = require('fs')
const nodeCrypto = require('crypto')
const zlib = require('zlib')

const cryptoJsPath = path.resolve(process.argv[2] || '../api-enhanced/util/crypto.js')
const out = path.resolve(__dirname, '../tests/vectors.json')

// ---- deterministic randomness
let randomQueue = []
nodeCrypto.randomBytes = (n) => {
  const b = randomQueue.shift()
  if (!b || b.length !== n) throw new Error('random queue mismatch for ' + n)
  return b
}
const seq = (n, start) => Buffer.from(Array.from({ length: n }, (_, i) => (start + i * 5) & 0xff))
const x25519Priv = (seed) =>
  nodeCrypto.createPrivateKey({
    key: Buffer.concat([Buffer.from('302e020100300506032b656e04220420', 'hex'), seed]),
    format: 'der',
    type: 'pkcs8',
  })
const rawPub = (priv) =>
  nodeCrypto.createPublicKey(priv).export({ format: 'der', type: 'spki' }).subarray(-32)
let ephSeed = null
nodeCrypto.generateKeyPairSync = () => {
  const privateKey = x25519Priv(ephSeed)
  return { privateKey, publicKey: nodeCrypto.createPublicKey(privateKey) }
}
const realRandom = Math.random
const enc = require(cryptoJsPath)

const v = {}

// ---- weapi
{
  const text = JSON.stringify({ ids: '[347230]', csrf_token: '', note: '中文 test' })
  const idx = Array.from({ length: 16 }, (_, i) => i)
  let i = 0
  Math.random = () => idx[i++] / 61
  const r = enc.weapi(JSON.parse(text))
  Math.random = realRandom
  v.weapi = { text, secret: 'abcdefghijklmnop', params: r.params, encSecKey: r.encSecKey }
}

// ---- eapi
{
  const url = '/api/song/enhance/player/url/v1'
  const text = JSON.stringify({ ids: '[347230]', level: 'exhigh', header: { os: 'iPhone OS' } })
  v.eapi = { url, text, params: enc.eapi(url, text).params }
}

// ---- eapi / xeapi response decrypt
{
  const ecb = (buf) => {
    const c = nodeCrypto.createCipheriv('aes-128-ecb', Buffer.from('e82ckenh8dichen8'), Buffer.alloc(0))
    return Buffer.concat([c.update(buf), c.final()])
  }
  const body = JSON.stringify({ code: 200, data: [{ id: 347230, url: 'https://x/y.mp3' }] })
  v.eapi_res = {
    plain: body,
    enc_hex: ecb(Buffer.from(body)).toString('hex'),
    gz_enc_hex: ecb(zlib.gzipSync(Buffer.from(body))).toString('hex'),
  }
  v.xeapi_res = { plain: body, enc_hex: v.eapi_res.enc_hex, gz_enc_hex: v.eapi_res.gz_enc_hex }
}

// ---- xeapi sign + public key blob
{
  v.sign = { ts: '1763200000000', nonce: '0123456789012345', sig: enc.xeapiSign('1763200000000', '0123456789012345') }
  const staticKey = Buffer.from('ab1d5a430f6bb04a3f01e81ddd72bd916d5ce591248ac128714806d7f8fb1b84', 'hex')
  const peerPriv = x25519Priv(seq(32, 100))
  const pub = {
    publicKey: rawPub(peerPriv).toString('base64'),
    version: 'v7',
    sk: 'SK-TEST-123',
  }
  const c = nodeCrypto.createCipheriv('aes-256-ecb', staticKey, Buffer.alloc(0))
  const blob = Buffer.concat([c.update(Buffer.from(JSON.stringify(pub))), c.final()]).toString('base64')
  v.public_key = { blob, decoded: pub }
}

// ---- xeapi requests
const xeapiCase = (name, uri, data, opts) => {
  const dyn = seq(16, 1), mid = seq(16, 40), iv = seq(12, 77)
  ephSeed = seq(32, 9)
  randomQueue = opts.sessionKey ? [mid, iv] : [dyn, mid, iv]
  const r = enc.xeapi(uri, { ...data }, { ...opts, publicKeyState: v.public_key.decoded })
  v[name] = {
    uri,
    body: Object.entries(data).map(([k, val]) => [k, String(val)]),
    opts: { os: opts.os || 'android', sessionKey: opts.sessionKey || null, sessionId: opts.sessionId || '' },
    rand: { dynamic_key: dyn.toString('hex'), mid_random: mid.toString('hex'), eph_secret: ephSeed.toString('hex'), gcm_iv: iv.toString('hex') },
    public_key: v.public_key.decoded,
    C: r.C, S: r.S, R: r.R,
  }
}
xeapiCase('xeapi_basic', '/api/song/enhance/player/url/v1',
  { ids: '[347230]', level: 'exhigh', encodeType: 'flac', e_r: false }, { os: 'android' })
xeapiCase('xeapi_unicode', '/api/register/anonimous?x=1',
  { username: 'abc==', keyword: '周杰伦 test&=+', e_r: false }, { os: 'android' })
xeapiCase('xeapi_session', '/api/song/enhance/player/url/v1',
  { ids: '[1]', level: 'standard' }, { os: 'android', sessionKey: '0123456789abcdef', sessionId: 'sess-1' })

fs.writeFileSync(out, JSON.stringify(v, null, 2))
console.log('wrote', out)
