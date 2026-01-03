var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// .wrangler/tmp/bundle-vPt2v8/checked-fetch.js
var urls = /* @__PURE__ */ new Set();
function checkURL(request, init) {
  const url = request instanceof URL ? request : new URL(
    (typeof request === "string" ? new Request(request, init) : request).url
  );
  if (url.port && url.port !== "443" && url.protocol === "https:") {
    if (!urls.has(url.toString())) {
      urls.add(url.toString());
      console.warn(
        `WARNING: known issue with \`fetch()\` requests to custom HTTPS ports in published Workers:
 - ${url.toString()} - the custom port will be ignored when the Worker is published using the \`wrangler deploy\` command.
`
      );
    }
  }
}
__name(checkURL, "checkURL");
globalThis.fetch = new Proxy(globalThis.fetch, {
  apply(target, thisArg, argArray) {
    const [request, init] = argArray;
    checkURL(request, init);
    return Reflect.apply(target, thisArg, argArray);
  }
});

// build/index.js
import { WorkerEntrypoint as ft } from "cloudflare:workers";
import D from "./f9cc836b383ed7800443c856436244d4f05f71c8-index_bg.wasm";
var _;
function w(t2) {
  let e = _.__externref_table_alloc();
  return _.__wbindgen_externrefs.set(e, t2), e;
}
__name(w, "w");
var L = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry((t2) => {
  t2.instance === o && t2.dtor(t2.a, t2.b);
});
function M(t2) {
  let e = typeof t2;
  if (e == "number" || e == "boolean" || t2 == null) return `${t2}`;
  if (e == "string") return `"${t2}"`;
  if (e == "symbol") {
    let i = t2.description;
    return i == null ? "Symbol" : `Symbol(${i})`;
  }
  if (e == "function") {
    let i = t2.name;
    return typeof i == "string" && i.length > 0 ? `Function(${i})` : "Function";
  }
  if (Array.isArray(t2)) {
    let i = t2.length, b = "[";
    i > 0 && (b += M(t2[0]));
    for (let c = 1; c < i; c++) b += ", " + M(t2[c]);
    return b += "]", b;
  }
  let n = /\[object ([^\]]+)\]/.exec(toString.call(t2)), r;
  if (n && n.length > 1) r = n[1];
  else return toString.call(t2);
  if (r == "Object") try {
    return "Object(" + JSON.stringify(t2) + ")";
  } catch {
    return "Object";
  }
  return t2 instanceof Error ? `${t2.name}: ${t2.message}
${t2.stack}` : r;
}
__name(M, "M");
function G(t2, e) {
  t2 = t2 >>> 0;
  let n = a(), r = [];
  for (let i = t2; i < t2 + 4 * e; i += 4) r.push(_.__wbindgen_externrefs.get(n.getUint32(i, true)));
  return _.__externref_drop_slice(t2, e), r;
}
__name(G, "G");
function q(t2, e) {
  return t2 = t2 >>> 0, S().subarray(t2 / 1, t2 / 1 + e);
}
__name(q, "q");
var p = null;
function a() {
  return (p === null || p.buffer.detached === true || p.buffer.detached === void 0 && p.buffer !== _.memory.buffer) && (p = new DataView(_.memory.buffer)), p;
}
__name(a, "a");
function g(t2, e) {
  return t2 = t2 >>> 0, X(t2, e);
}
__name(g, "g");
var j = null;
function S() {
  return (j === null || j.byteLength === 0) && (j = new Uint8Array(_.memory.buffer)), j;
}
__name(S, "S");
function f(t2, e) {
  try {
    return t2.apply(this, e);
  } catch (n) {
    let r = w(n);
    _.__wbindgen_exn_store(r);
  }
}
__name(f, "f");
function s(t2) {
  return t2 == null;
}
__name(s, "s");
function K(t2, e, n, r) {
  let i = { a: t2, b: e, cnt: 1, dtor: n, instance: o }, b = /* @__PURE__ */ __name((...c) => {
    if (i.instance !== o) throw new Error("Cannot invoke closure from previous WASM instance");
    i.cnt++;
    let u = i.a;
    i.a = 0;
    try {
      return r(u, i.b, ...c);
    } finally {
      i.a = u, b._wbg_cb_unref();
    }
  }, "b");
  return b._wbg_cb_unref = () => {
    --i.cnt === 0 && (i.dtor(i.a, i.b), i.a = 0, L.unregister(i));
  }, L.register(b, i, i), b;
}
__name(K, "K");
function Q(t2, e) {
  let n = e(t2.length * 4, 4) >>> 0;
  for (let r = 0; r < t2.length; r++) {
    let i = w(t2[r]);
    a().setUint32(n + 4 * r, i, true);
  }
  return d = t2.length, n;
}
__name(Q, "Q");
function m(t2, e, n) {
  if (n === void 0) {
    let u = W.encode(t2), l = e(u.length, 1) >>> 0;
    return S().subarray(l, l + u.length).set(u), d = u.length, l;
  }
  let r = t2.length, i = e(r, 1) >>> 0, b = S(), c = 0;
  for (; c < r; c++) {
    let u = t2.charCodeAt(c);
    if (u > 127) break;
    b[i + c] = u;
  }
  if (c !== r) {
    c !== 0 && (t2 = t2.slice(c)), i = n(i, r, r = c + t2.length * 3, 1) >>> 0;
    let u = S().subarray(i + c, i + r), l = W.encodeInto(t2, u);
    c += l.written, i = n(i, r, c, 1) >>> 0;
  }
  return d = c, i;
}
__name(m, "m");
var H = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
H.decode();
function X(t2, e) {
  return H.decode(S().subarray(t2, t2 + e));
}
__name(X, "X");
var W = new TextEncoder();
"encodeInto" in W || (W.encodeInto = function(t2, e) {
  let n = W.encode(t2);
  return e.set(n), { read: t2.length, written: n.length };
});
var d = 0;
function Y(t2, e, n) {
  _.wasm_bindgen__convert__closures_____invoke__hadd62ad0a0add888(t2, e, n);
}
__name(Y, "Y");
function Z(t2, e, n, r) {
  _.wasm_bindgen__convert__closures_____invoke__h1b7b6a79ae010c01(t2, e, n, r);
}
__name(Z, "Z");
var tt = ["bytes"];
var et = ["follow", "error", "manual"];
var o = 0;
var nt = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_containerstartupoptions_free(t2 >>> 0, 1);
});
var rt = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_intounderlyingbytesource_free(t2 >>> 0, 1);
});
var _t = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_intounderlyingsink_free(t2 >>> 0, 1);
});
var it = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_intounderlyingsource_free(t2 >>> 0, 1);
});
var B = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_minifyconfig_free(t2 >>> 0, 1);
});
var ot = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: t2, instance: e }) => {
  e === o && _.__wbg_r2range_free(t2 >>> 0, 1);
});
var v = class {
  static {
    __name(this, "v");
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, nt.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_containerstartupoptions_free(e, 0);
  }
  get entrypoint() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.__wbg_get_containerstartupoptions_entrypoint(this.__wbg_ptr);
    var n = G(e[0], e[1]).slice();
    return _.__wbindgen_free(e[0], e[1] * 4, 4), n;
  }
  set entrypoint(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let n = Q(e, _.__wbindgen_malloc), r = d;
    _.__wbg_set_containerstartupoptions_entrypoint(this.__wbg_ptr, n, r);
  }
  get enableInternet() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.__wbg_get_containerstartupoptions_enableInternet(this.__wbg_ptr);
    return e === 16777215 ? void 0 : e !== 0;
  }
  set enableInternet(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_containerstartupoptions_enableInternet(this.__wbg_ptr, s(e) ? 16777215 : e ? 1 : 0);
  }
  get env() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.__wbg_get_containerstartupoptions_env(this.__wbg_ptr);
  }
  set env(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_containerstartupoptions_env(this.__wbg_ptr, e);
  }
};
Symbol.dispose && (v.prototype[Symbol.dispose] = v.prototype.free);
var x = class {
  static {
    __name(this, "x");
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, rt.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_intounderlyingbytesource_free(e, 0);
  }
  get autoAllocateChunkSize() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr) >>> 0;
  }
  pull(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.intounderlyingbytesource_pull(this.__wbg_ptr, e);
  }
  start(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.intounderlyingbytesource_start(this.__wbg_ptr, e);
  }
  get type() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.intounderlyingbytesource_type(this.__wbg_ptr);
    return tt[e];
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = this.__destroy_into_raw();
    _.intounderlyingbytesource_cancel(e);
  }
};
Symbol.dispose && (x.prototype[Symbol.dispose] = x.prototype.free);
var I = class {
  static {
    __name(this, "I");
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, _t.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_intounderlyingsink_free(e, 0);
  }
  abort(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let n = this.__destroy_into_raw();
    return _.intounderlyingsink_abort(n, e);
  }
  close() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = this.__destroy_into_raw();
    return _.intounderlyingsink_close(e);
  }
  write(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.intounderlyingsink_write(this.__wbg_ptr, e);
  }
};
Symbol.dispose && (I.prototype[Symbol.dispose] = I.prototype.free);
var E = class {
  static {
    __name(this, "E");
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, it.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_intounderlyingsource_free(e, 0);
  }
  pull(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.intounderlyingsource_pull(this.__wbg_ptr, e);
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = this.__destroy_into_raw();
    _.intounderlyingsource_cancel(e);
  }
};
Symbol.dispose && (E.prototype[Symbol.dispose] = E.prototype.free);
var h = class t {
  static {
    __name(this, "t");
  }
  static __wrap(e) {
    e = e >>> 0;
    let n = Object.create(t.prototype);
    return n.__wbg_ptr = e, n.__wbg_inst = o, B.register(n, { ptr: e, instance: o }, n), n;
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, B.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_minifyconfig_free(e, 0);
  }
  get js() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.__wbg_get_minifyconfig_js(this.__wbg_ptr) !== 0;
  }
  set js(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_minifyconfig_js(this.__wbg_ptr, e);
  }
  get html() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.__wbg_get_minifyconfig_html(this.__wbg_ptr) !== 0;
  }
  set html(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_minifyconfig_html(this.__wbg_ptr, e);
  }
  get css() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    return _.__wbg_get_minifyconfig_css(this.__wbg_ptr) !== 0;
  }
  set css(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_minifyconfig_css(this.__wbg_ptr, e);
  }
};
Symbol.dispose && (h.prototype[Symbol.dispose] = h.prototype.free);
var R = class {
  static {
    __name(this, "R");
  }
  __destroy_into_raw() {
    let e = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ot.unregister(this), e;
  }
  free() {
    let e = this.__destroy_into_raw();
    _.__wbg_r2range_free(e, 0);
  }
  get offset() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.__wbg_get_r2range_offset(this.__wbg_ptr);
    return e[0] === 0 ? void 0 : e[1];
  }
  set offset(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_r2range_offset(this.__wbg_ptr, !s(e), s(e) ? 0 : e);
  }
  get length() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.__wbg_get_r2range_length(this.__wbg_ptr);
    return e[0] === 0 ? void 0 : e[1];
  }
  set length(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_r2range_length(this.__wbg_ptr, !s(e), s(e) ? 0 : e);
  }
  get suffix() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    let e = _.__wbg_get_r2range_suffix(this.__wbg_ptr);
    return e[0] === 0 ? void 0 : e[1];
  }
  set suffix(e) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== o) throw new Error("Invalid stale object from previous Wasm instance");
    _.__wbg_set_r2range_suffix(this.__wbg_ptr, !s(e), s(e) ? 0 : e);
  }
};
Symbol.dispose && (R.prototype[Symbol.dispose] = R.prototype.free);
function N() {
  o++, p = null, j = null, typeof numBytesDecoded < "u" && (numBytesDecoded = 0), typeof d < "u" && (d = 0), _ = new WebAssembly.Instance(D, V).exports, _.__wbindgen_start();
}
__name(N, "N");
function $(t2, e, n) {
  return _.fetch(t2, e, n);
}
__name($, "$");
function U(t2) {
  _.setPanicHook(t2);
}
__name(U, "U");
var V = { __wbindgen_placeholder__: { __wbg_Error_52673b7de5a0ca89: /* @__PURE__ */ __name(function(t2, e) {
  return Error(g(t2, e));
}, "__wbg_Error_52673b7de5a0ca89"), __wbg_String_8f0eb39a4a4c2f66: /* @__PURE__ */ __name(function(t2, e) {
  let n = String(e), r = m(n, _.__wbindgen_malloc, _.__wbindgen_realloc), i = d;
  a().setInt32(t2 + 4, i, true), a().setInt32(t2 + 0, r, true);
}, "__wbg_String_8f0eb39a4a4c2f66"), __wbg___wbindgen_bigint_get_as_i64_6e32f5e6aff02e1d: /* @__PURE__ */ __name(function(t2, e) {
  let n = e, r = typeof n == "bigint" ? n : void 0;
  a().setBigInt64(t2 + 8, s(r) ? BigInt(0) : r, true), a().setInt32(t2 + 0, !s(r), true);
}, "__wbg___wbindgen_bigint_get_as_i64_6e32f5e6aff02e1d"), __wbg___wbindgen_boolean_get_dea25b33882b895b: /* @__PURE__ */ __name(function(t2) {
  let e = t2, n = typeof e == "boolean" ? e : void 0;
  return s(n) ? 16777215 : n ? 1 : 0;
}, "__wbg___wbindgen_boolean_get_dea25b33882b895b"), __wbg___wbindgen_debug_string_adfb662ae34724b6: /* @__PURE__ */ __name(function(t2, e) {
  let n = M(e), r = m(n, _.__wbindgen_malloc, _.__wbindgen_realloc), i = d;
  a().setInt32(t2 + 4, i, true), a().setInt32(t2 + 0, r, true);
}, "__wbg___wbindgen_debug_string_adfb662ae34724b6"), __wbg___wbindgen_in_0d3e1e8f0c669317: /* @__PURE__ */ __name(function(t2, e) {
  return t2 in e;
}, "__wbg___wbindgen_in_0d3e1e8f0c669317"), __wbg___wbindgen_is_bigint_0e1a2e3f55cfae27: /* @__PURE__ */ __name(function(t2) {
  return typeof t2 == "bigint";
}, "__wbg___wbindgen_is_bigint_0e1a2e3f55cfae27"), __wbg___wbindgen_is_function_8d400b8b1af978cd: /* @__PURE__ */ __name(function(t2) {
  return typeof t2 == "function";
}, "__wbg___wbindgen_is_function_8d400b8b1af978cd"), __wbg___wbindgen_is_object_ce774f3490692386: /* @__PURE__ */ __name(function(t2) {
  let e = t2;
  return typeof e == "object" && e !== null;
}, "__wbg___wbindgen_is_object_ce774f3490692386"), __wbg___wbindgen_is_string_704ef9c8fc131030: /* @__PURE__ */ __name(function(t2) {
  return typeof t2 == "string";
}, "__wbg___wbindgen_is_string_704ef9c8fc131030"), __wbg___wbindgen_is_undefined_f6b95eab589e0269: /* @__PURE__ */ __name(function(t2) {
  return t2 === void 0;
}, "__wbg___wbindgen_is_undefined_f6b95eab589e0269"), __wbg___wbindgen_jsval_eq_b6101cc9cef1fe36: /* @__PURE__ */ __name(function(t2, e) {
  return t2 === e;
}, "__wbg___wbindgen_jsval_eq_b6101cc9cef1fe36"), __wbg___wbindgen_jsval_loose_eq_766057600fdd1b0d: /* @__PURE__ */ __name(function(t2, e) {
  return t2 == e;
}, "__wbg___wbindgen_jsval_loose_eq_766057600fdd1b0d"), __wbg___wbindgen_number_get_9619185a74197f95: /* @__PURE__ */ __name(function(t2, e) {
  let n = e, r = typeof n == "number" ? n : void 0;
  a().setFloat64(t2 + 8, s(r) ? 0 : r, true), a().setInt32(t2 + 0, !s(r), true);
}, "__wbg___wbindgen_number_get_9619185a74197f95"), __wbg___wbindgen_string_get_a2a31e16edf96e42: /* @__PURE__ */ __name(function(t2, e) {
  let n = e, r = typeof n == "string" ? n : void 0;
  var i = s(r) ? 0 : m(r, _.__wbindgen_malloc, _.__wbindgen_realloc), b = d;
  a().setInt32(t2 + 4, b, true), a().setInt32(t2 + 0, i, true);
}, "__wbg___wbindgen_string_get_a2a31e16edf96e42"), __wbg___wbindgen_throw_dd24417ed36fc46e: /* @__PURE__ */ __name(function(t2, e) {
  throw new Error(g(t2, e));
}, "__wbg___wbindgen_throw_dd24417ed36fc46e"), __wbg__wbg_cb_unref_87dfb5aaa0cbcea7: /* @__PURE__ */ __name(function(t2) {
  t2._wbg_cb_unref();
}, "__wbg__wbg_cb_unref_87dfb5aaa0cbcea7"), __wbg_body_947b901c33f7fe32: /* @__PURE__ */ __name(function(t2) {
  let e = t2.body;
  return s(e) ? 0 : w(e);
}, "__wbg_body_947b901c33f7fe32"), __wbg_buffer_6cb2fecb1f253d71: /* @__PURE__ */ __name(function(t2) {
  return t2.buffer;
}, "__wbg_buffer_6cb2fecb1f253d71"), __wbg_byobRequest_f8e3517f5f8ad284: /* @__PURE__ */ __name(function(t2) {
  let e = t2.byobRequest;
  return s(e) ? 0 : w(e);
}, "__wbg_byobRequest_f8e3517f5f8ad284"), __wbg_byteLength_faa9938885bdeee6: /* @__PURE__ */ __name(function(t2) {
  return t2.byteLength;
}, "__wbg_byteLength_faa9938885bdeee6"), __wbg_byteOffset_3868b6a19ba01dea: /* @__PURE__ */ __name(function(t2) {
  return t2.byteOffset;
}, "__wbg_byteOffset_3868b6a19ba01dea"), __wbg_call_3020136f7a2d6e44: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n) {
    return t2.call(e, n);
  }, arguments);
}, "__wbg_call_3020136f7a2d6e44"), __wbg_call_abb4ff46ce38be40: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return t2.call(e);
  }, arguments);
}, "__wbg_call_abb4ff46ce38be40"), __wbg_cause_2863fe79d084e5de: /* @__PURE__ */ __name(function(t2) {
  return t2.cause;
}, "__wbg_cause_2863fe79d084e5de"), __wbg_cf_34056ec69704ac68: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    let e = t2.cf;
    return s(e) ? 0 : w(e);
  }, arguments);
}, "__wbg_cf_34056ec69704ac68"), __wbg_cf_90e0ec4ff8f9a6fc: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    let e = t2.cf;
    return s(e) ? 0 : w(e);
  }, arguments);
}, "__wbg_cf_90e0ec4ff8f9a6fc"), __wbg_close_0af5661bf3d335f2: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    t2.close();
  }, arguments);
}, "__wbg_close_0af5661bf3d335f2"), __wbg_close_3ec111e7b23d94d8: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    t2.close();
  }, arguments);
}, "__wbg_close_3ec111e7b23d94d8"), __wbg_constructor_bd34b914d5a3a404: /* @__PURE__ */ __name(function(t2) {
  return t2.constructor;
}, "__wbg_constructor_bd34b914d5a3a404"), __wbg_done_62ea16af4ce34b24: /* @__PURE__ */ __name(function(t2) {
  return t2.done;
}, "__wbg_done_62ea16af4ce34b24"), __wbg_enqueue_a7e6b1ee87963aad: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    t2.enqueue(e);
  }, arguments);
}, "__wbg_enqueue_a7e6b1ee87963aad"), __wbg_entries_83c79938054e065f: /* @__PURE__ */ __name(function(t2) {
  return Object.entries(t2);
}, "__wbg_entries_83c79938054e065f"), __wbg_error_7534b8e9a36f1ab4: /* @__PURE__ */ __name(function(t2, e) {
  let n, r;
  try {
    n = t2, r = e, console.error(g(t2, e));
  } finally {
    _.__wbindgen_free(n, r, 1);
  }
}, "__wbg_error_7534b8e9a36f1ab4"), __wbg_error_7bc7d576a6aaf855: /* @__PURE__ */ __name(function(t2) {
  console.error(t2);
}, "__wbg_error_7bc7d576a6aaf855"), __wbg_error_d7f117185d9ffd19: /* @__PURE__ */ __name(function(t2, e) {
  console.error(t2, e);
}, "__wbg_error_d7f117185d9ffd19"), __wbg_fetch_0e71ea1ff4e415db: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return t2.fetch(e);
  }, arguments);
}, "__wbg_fetch_0e71ea1ff4e415db"), __wbg_get_6b7bd52aca3f9671: /* @__PURE__ */ __name(function(t2, e) {
  return t2[e >>> 0];
}, "__wbg_get_6b7bd52aca3f9671"), __wbg_get_850c4f39a577ec31: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return t2.get(e);
  }, arguments);
}, "__wbg_get_850c4f39a577ec31"), __wbg_get_af9dab7e9603ea93: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return Reflect.get(t2, e);
  }, arguments);
}, "__wbg_get_af9dab7e9603ea93"), __wbg_headers_654c30e1bcccc552: /* @__PURE__ */ __name(function(t2) {
  return t2.headers;
}, "__wbg_headers_654c30e1bcccc552"), __wbg_headers_850c3fb50632ae78: /* @__PURE__ */ __name(function(t2) {
  return t2.headers;
}, "__wbg_headers_850c3fb50632ae78"), __wbg_idFromName_46c84e3f60ef66ec: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n) {
    return t2.idFromName(g(e, n));
  }, arguments);
}, "__wbg_idFromName_46c84e3f60ef66ec"), __wbg_instanceof_ArrayBuffer_f3320d2419cd0355: /* @__PURE__ */ __name(function(t2) {
  let e;
  try {
    e = t2 instanceof ArrayBuffer;
  } catch {
    e = false;
  }
  return e;
}, "__wbg_instanceof_ArrayBuffer_f3320d2419cd0355"), __wbg_instanceof_Error_3443650560328fa9: /* @__PURE__ */ __name(function(t2) {
  let e;
  try {
    e = t2 instanceof Error;
  } catch {
    e = false;
  }
  return e;
}, "__wbg_instanceof_Error_3443650560328fa9"), __wbg_instanceof_Map_084be8da74364158: /* @__PURE__ */ __name(function(t2) {
  let e;
  try {
    e = t2 instanceof Map;
  } catch {
    e = false;
  }
  return e;
}, "__wbg_instanceof_Map_084be8da74364158"), __wbg_instanceof_Response_cd74d1c2ac92cb0b: /* @__PURE__ */ __name(function(t2) {
  let e;
  try {
    e = t2 instanceof Response;
  } catch {
    e = false;
  }
  return e;
}, "__wbg_instanceof_Response_cd74d1c2ac92cb0b"), __wbg_instanceof_Uint8Array_da54ccc9d3e09434: /* @__PURE__ */ __name(function(t2) {
  let e;
  try {
    e = t2 instanceof Uint8Array;
  } catch {
    e = false;
  }
  return e;
}, "__wbg_instanceof_Uint8Array_da54ccc9d3e09434"), __wbg_isArray_51fd9e6422c0a395: /* @__PURE__ */ __name(function(t2) {
  return Array.isArray(t2);
}, "__wbg_isArray_51fd9e6422c0a395"), __wbg_isSafeInteger_ae7d3f054d55fa16: /* @__PURE__ */ __name(function(t2) {
  return Number.isSafeInteger(t2);
}, "__wbg_isSafeInteger_ae7d3f054d55fa16"), __wbg_iterator_27b7c8b35ab3e86b: /* @__PURE__ */ __name(function() {
  return Symbol.iterator;
}, "__wbg_iterator_27b7c8b35ab3e86b"), __wbg_json_84cfc63d751277ef: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    return t2.json();
  }, arguments);
}, "__wbg_json_84cfc63d751277ef"), __wbg_length_22ac23eaec9d8053: /* @__PURE__ */ __name(function(t2) {
  return t2.length;
}, "__wbg_length_22ac23eaec9d8053"), __wbg_length_d45040a40c570362: /* @__PURE__ */ __name(function(t2) {
  return t2.length;
}, "__wbg_length_d45040a40c570362"), __wbg_log_1d990106d99dacb7: /* @__PURE__ */ __name(function(t2) {
  console.log(t2);
}, "__wbg_log_1d990106d99dacb7"), __wbg_method_6a1f0d0a9e501984: /* @__PURE__ */ __name(function(t2, e) {
  let n = e.method, r = m(n, _.__wbindgen_malloc, _.__wbindgen_realloc), i = d;
  a().setInt32(t2 + 4, i, true), a().setInt32(t2 + 0, r, true);
}, "__wbg_method_6a1f0d0a9e501984"), __wbg_minifyconfig_new: /* @__PURE__ */ __name(function(t2) {
  return h.__wrap(t2);
}, "__wbg_minifyconfig_new"), __wbg_name_6d8c704cecb9e350: /* @__PURE__ */ __name(function(t2) {
  return t2.name;
}, "__wbg_name_6d8c704cecb9e350"), __wbg_new_1ba21ce319a06297: /* @__PURE__ */ __name(function() {
  return new Object();
}, "__wbg_new_1ba21ce319a06297"), __wbg_new_3c79b3bb1b32b7d3: /* @__PURE__ */ __name(function() {
  return f(function() {
    return new Headers();
  }, arguments);
}, "__wbg_new_3c79b3bb1b32b7d3"), __wbg_new_6421f6084cc5bc5a: /* @__PURE__ */ __name(function(t2) {
  return new Uint8Array(t2);
}, "__wbg_new_6421f6084cc5bc5a"), __wbg_new_8a6f238a6ece86ea: /* @__PURE__ */ __name(function() {
  return new Error();
}, "__wbg_new_8a6f238a6ece86ea"), __wbg_new_b546ae120718850e: /* @__PURE__ */ __name(function() {
  return /* @__PURE__ */ new Map();
}, "__wbg_new_b546ae120718850e"), __wbg_new_df1173567d5ff028: /* @__PURE__ */ __name(function(t2, e) {
  return new Error(g(t2, e));
}, "__wbg_new_df1173567d5ff028"), __wbg_new_ff12d2b041fb48f1: /* @__PURE__ */ __name(function(t2, e) {
  try {
    var n = { a: t2, b: e }, r = /* @__PURE__ */ __name((b, c) => {
      let u = n.a;
      n.a = 0;
      try {
        return Z(u, n.b, b, c);
      } finally {
        n.a = u;
      }
    }, "r");
    return new Promise(r);
  } finally {
    n.a = n.b = 0;
  }
}, "__wbg_new_ff12d2b041fb48f1"), __wbg_new_no_args_cb138f77cf6151ee: /* @__PURE__ */ __name(function(t2, e) {
  return new Function(g(t2, e));
}, "__wbg_new_no_args_cb138f77cf6151ee"), __wbg_new_with_byte_offset_and_length_d85c3da1fd8df149: /* @__PURE__ */ __name(function(t2, e, n) {
  return new Uint8Array(t2, e >>> 0, n >>> 0);
}, "__wbg_new_with_byte_offset_and_length_d85c3da1fd8df149"), __wbg_new_with_length_aa5eaf41d35235e5: /* @__PURE__ */ __name(function(t2) {
  return new Uint8Array(t2 >>> 0);
}, "__wbg_new_with_length_aa5eaf41d35235e5"), __wbg_new_with_opt_buffer_source_and_init_1200e907bc1ec81d: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return new Response(t2, e);
  }, arguments);
}, "__wbg_new_with_opt_buffer_source_and_init_1200e907bc1ec81d"), __wbg_new_with_opt_readable_stream_and_init_6377f53b425fda23: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    return new Response(t2, e);
  }, arguments);
}, "__wbg_new_with_opt_readable_stream_and_init_6377f53b425fda23"), __wbg_new_with_opt_str_and_init_01a4a75000df79cb: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n) {
    return new Response(t2 === 0 ? void 0 : g(t2, e), n);
  }, arguments);
}, "__wbg_new_with_opt_str_and_init_01a4a75000df79cb"), __wbg_new_with_str_and_init_c5748f76f5108934: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n) {
    return new Request(g(t2, e), n);
  }, arguments);
}, "__wbg_new_with_str_and_init_c5748f76f5108934"), __wbg_next_138a17bbf04e926c: /* @__PURE__ */ __name(function(t2) {
  return t2.next;
}, "__wbg_next_138a17bbf04e926c"), __wbg_next_3cfe5c0fe2a4cc53: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    return t2.next();
  }, arguments);
}, "__wbg_next_3cfe5c0fe2a4cc53"), __wbg_now_69d776cd24f5215b: /* @__PURE__ */ __name(function() {
  return Date.now();
}, "__wbg_now_69d776cd24f5215b"), __wbg_prototypesetcall_dfe9b766cdc1f1fd: /* @__PURE__ */ __name(function(t2, e, n) {
  Uint8Array.prototype.set.call(q(t2, e), n);
}, "__wbg_prototypesetcall_dfe9b766cdc1f1fd"), __wbg_queueMicrotask_9b549dfce8865860: /* @__PURE__ */ __name(function(t2) {
  return t2.queueMicrotask;
}, "__wbg_queueMicrotask_9b549dfce8865860"), __wbg_queueMicrotask_fca69f5bfad613a5: /* @__PURE__ */ __name(function(t2) {
  queueMicrotask(t2);
}, "__wbg_queueMicrotask_fca69f5bfad613a5"), __wbg_resolve_fd5bfbaa4ce36e1e: /* @__PURE__ */ __name(function(t2) {
  return Promise.resolve(t2);
}, "__wbg_resolve_fd5bfbaa4ce36e1e"), __wbg_respond_9f7fc54636c4a3af: /* @__PURE__ */ __name(function() {
  return f(function(t2, e) {
    t2.respond(e >>> 0);
  }, arguments);
}, "__wbg_respond_9f7fc54636c4a3af"), __wbg_set_169e13b608078b7b: /* @__PURE__ */ __name(function(t2, e, n) {
  t2.set(q(e, n));
}, "__wbg_set_169e13b608078b7b"), __wbg_set_3f1d0b984ed272ed: /* @__PURE__ */ __name(function(t2, e, n) {
  t2[e] = n;
}, "__wbg_set_3f1d0b984ed272ed"), __wbg_set_425eb8b710d5beee: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n, r, i) {
    t2.set(g(e, n), g(r, i));
  }, arguments);
}, "__wbg_set_425eb8b710d5beee"), __wbg_set_781438a03c0c3c81: /* @__PURE__ */ __name(function() {
  return f(function(t2, e, n) {
    return Reflect.set(t2, e, n);
  }, arguments);
}, "__wbg_set_781438a03c0c3c81"), __wbg_set_body_8e743242d6076a4f: /* @__PURE__ */ __name(function(t2, e) {
  t2.body = e;
}, "__wbg_set_body_8e743242d6076a4f"), __wbg_set_efaaf145b9377369: /* @__PURE__ */ __name(function(t2, e, n) {
  return t2.set(e, n);
}, "__wbg_set_efaaf145b9377369"), __wbg_set_headers_5671cf088e114d2b: /* @__PURE__ */ __name(function(t2, e) {
  t2.headers = e;
}, "__wbg_set_headers_5671cf088e114d2b"), __wbg_set_headers_9f734278b4257b03: /* @__PURE__ */ __name(function(t2, e) {
  t2.headers = e;
}, "__wbg_set_headers_9f734278b4257b03"), __wbg_set_method_76c69e41b3570627: /* @__PURE__ */ __name(function(t2, e, n) {
  t2.method = g(e, n);
}, "__wbg_set_method_76c69e41b3570627"), __wbg_set_redirect_e125c2dc00f1a7bf: /* @__PURE__ */ __name(function(t2, e) {
  t2.redirect = et[e];
}, "__wbg_set_redirect_e125c2dc00f1a7bf"), __wbg_set_status_2727ed43f6735170: /* @__PURE__ */ __name(function(t2, e) {
  t2.status = e;
}, "__wbg_set_status_2727ed43f6735170"), __wbg_stack_0ed75d68575b0f3c: /* @__PURE__ */ __name(function(t2, e) {
  let n = e.stack, r = m(n, _.__wbindgen_malloc, _.__wbindgen_realloc), i = d;
  a().setInt32(t2 + 4, i, true), a().setInt32(t2 + 0, r, true);
}, "__wbg_stack_0ed75d68575b0f3c"), __wbg_static_accessor_GLOBAL_769e6b65d6557335: /* @__PURE__ */ __name(function() {
  let t2 = typeof global > "u" ? null : global;
  return s(t2) ? 0 : w(t2);
}, "__wbg_static_accessor_GLOBAL_769e6b65d6557335"), __wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1: /* @__PURE__ */ __name(function() {
  let t2 = typeof globalThis > "u" ? null : globalThis;
  return s(t2) ? 0 : w(t2);
}, "__wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1"), __wbg_static_accessor_SELF_08f5a74c69739274: /* @__PURE__ */ __name(function() {
  let t2 = typeof self > "u" ? null : self;
  return s(t2) ? 0 : w(t2);
}, "__wbg_static_accessor_SELF_08f5a74c69739274"), __wbg_static_accessor_WINDOW_a8924b26aa92d024: /* @__PURE__ */ __name(function() {
  let t2 = typeof window > "u" ? null : window;
  return s(t2) ? 0 : w(t2);
}, "__wbg_static_accessor_WINDOW_a8924b26aa92d024"), __wbg_status_9bfc680efca4bdfd: /* @__PURE__ */ __name(function(t2) {
  return t2.status;
}, "__wbg_status_9bfc680efca4bdfd"), __wbg_then_429f7caf1026411d: /* @__PURE__ */ __name(function(t2, e, n) {
  return t2.then(e, n);
}, "__wbg_then_429f7caf1026411d"), __wbg_then_4f95312d68691235: /* @__PURE__ */ __name(function(t2, e) {
  return t2.then(e);
}, "__wbg_then_4f95312d68691235"), __wbg_toString_14b47ee7542a49ef: /* @__PURE__ */ __name(function(t2) {
  return t2.toString();
}, "__wbg_toString_14b47ee7542a49ef"), __wbg_url_87f30c96ceb3baf7: /* @__PURE__ */ __name(function(t2, e) {
  let n = e.url, r = m(n, _.__wbindgen_malloc, _.__wbindgen_realloc), i = d;
  a().setInt32(t2 + 4, i, true), a().setInt32(t2 + 0, r, true);
}, "__wbg_url_87f30c96ceb3baf7"), __wbg_value_57b7b035e117f7ee: /* @__PURE__ */ __name(function(t2) {
  return t2.value;
}, "__wbg_value_57b7b035e117f7ee"), __wbg_view_788aaf149deefd2f: /* @__PURE__ */ __name(function(t2) {
  let e = t2.view;
  return s(e) ? 0 : w(e);
}, "__wbg_view_788aaf149deefd2f"), __wbg_webSocket_a0b05dd767ed2a8a: /* @__PURE__ */ __name(function() {
  return f(function(t2) {
    let e = t2.webSocket;
    return s(e) ? 0 : w(e);
  }, arguments);
}, "__wbg_webSocket_a0b05dd767ed2a8a"), __wbindgen_cast_2241b6af4c4b2941: /* @__PURE__ */ __name(function(t2, e) {
  return g(t2, e);
}, "__wbindgen_cast_2241b6af4c4b2941"), __wbindgen_cast_4625c577ab2ec9ee: /* @__PURE__ */ __name(function(t2) {
  return BigInt.asUintN(64, t2);
}, "__wbindgen_cast_4625c577ab2ec9ee"), __wbindgen_cast_9ae0607507abb057: /* @__PURE__ */ __name(function(t2) {
  return t2;
}, "__wbindgen_cast_9ae0607507abb057"), __wbindgen_cast_b149864f50f9d2ff: /* @__PURE__ */ __name(function(t2, e) {
  return K(t2, e, _.wasm_bindgen__closure__destroy__ha5984c7f165cb418, Y);
}, "__wbindgen_cast_b149864f50f9d2ff"), __wbindgen_cast_d6cd19b81560fd6e: /* @__PURE__ */ __name(function(t2) {
  return t2;
}, "__wbindgen_cast_d6cd19b81560fd6e"), __wbindgen_init_externref_table: /* @__PURE__ */ __name(function() {
  let t2 = _.__wbindgen_externrefs, e = t2.grow(4);
  t2.set(0, void 0), t2.set(e + 0, void 0), t2.set(e + 1, null), t2.set(e + 2, true), t2.set(e + 3, false);
}, "__wbindgen_init_externref_table") } };
var st = new WebAssembly.Instance(D, V);
_ = st.exports;
_.__wbindgen_start();
Error.stackTraceLimit = 100;
var k = false;
function J() {
  U && U(function(t2) {
    let e = new Error("Rust panic: " + t2);
    console.error("Critical", e), k = true;
  });
}
__name(J, "J");
J();
var A = 0;
function T() {
  k && (console.log("Reinitializing Wasm application"), N(), k = false, J(), A++);
}
__name(T, "T");
addEventListener("error", (t2) => {
  C(t2.error);
});
function C(t2) {
  t2 instanceof WebAssembly.RuntimeError && (console.error("Critical", t2), k = true);
}
__name(C, "C");
var P = class extends ft {
  static {
    __name(this, "P");
  }
};
P.prototype.fetch = function(e) {
  return $.call(this, e, this.env, this.ctx);
};
var ut = { set: /* @__PURE__ */ __name((t2, e, n, r) => Reflect.set(t2.instance, e, n, r), "set"), has: /* @__PURE__ */ __name((t2, e) => Reflect.has(t2.instance, e), "has"), deleteProperty: /* @__PURE__ */ __name((t2, e) => Reflect.deleteProperty(t2.instance, e), "deleteProperty"), apply: /* @__PURE__ */ __name((t2, e, n) => Reflect.apply(t2.instance, e, n), "apply"), construct: /* @__PURE__ */ __name((t2, e, n) => Reflect.construct(t2.instance, e, n), "construct"), getPrototypeOf: /* @__PURE__ */ __name((t2) => Reflect.getPrototypeOf(t2.instance), "getPrototypeOf"), setPrototypeOf: /* @__PURE__ */ __name((t2, e) => Reflect.setPrototypeOf(t2.instance, e), "setPrototypeOf"), isExtensible: /* @__PURE__ */ __name((t2) => Reflect.isExtensible(t2.instance), "isExtensible"), preventExtensions: /* @__PURE__ */ __name((t2) => Reflect.preventExtensions(t2.instance), "preventExtensions"), getOwnPropertyDescriptor: /* @__PURE__ */ __name((t2, e) => Reflect.getOwnPropertyDescriptor(t2.instance, e), "getOwnPropertyDescriptor"), defineProperty: /* @__PURE__ */ __name((t2, e, n) => Reflect.defineProperty(t2.instance, e, n), "defineProperty"), ownKeys: /* @__PURE__ */ __name((t2) => Reflect.ownKeys(t2.instance), "ownKeys") };
var y = { construct(t2, e, n) {
  try {
    T();
    let r = { instance: Reflect.construct(t2, e, n), instanceId: A, ctor: t2, args: e, newTarget: n };
    return new Proxy(r, { ...ut, get(i, b, c) {
      i.instanceId !== A && (i.instance = Reflect.construct(i.ctor, i.args, i.newTarget), i.instanceId = A);
      let u = Reflect.get(i.instance, b, c);
      return typeof u != "function" ? u : u.constructor === Function ? new Proxy(u, { apply(l, z, O) {
        T();
        try {
          return l.apply(z, O);
        } catch (F) {
          throw C(F), F;
        }
      } }) : new Proxy(u, { async apply(l, z, O) {
        T();
        try {
          return await l.apply(z, O);
        } catch (F) {
          throw C(F), F;
        }
      } });
    } });
  } catch (r) {
    throw k = true, r;
  }
} };
var gt = new Proxy(P, y);
var wt = new Proxy(v, y);
var dt = new Proxy(x, y);
var lt = new Proxy(I, y);
var pt = new Proxy(E, y);
var ht = new Proxy(h, y);
var yt = new Proxy(R, y);

// ../../../../../../usr/lib/node_modules/wrangler/templates/middleware/middleware-ensure-req-body-drained.ts
var drainBody = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } finally {
    try {
      if (request.body !== null && !request.bodyUsed) {
        const reader = request.body.getReader();
        while (!(await reader.read()).done) {
        }
      }
    } catch (e) {
      console.error("Failed to drain the unused request body.", e);
    }
  }
}, "drainBody");
var middleware_ensure_req_body_drained_default = drainBody;

// ../../../../../../usr/lib/node_modules/wrangler/templates/middleware/middleware-miniflare3-json-error.ts
function reduceError(e) {
  return {
    name: e?.name,
    message: e?.message ?? String(e),
    stack: e?.stack,
    cause: e?.cause === void 0 ? void 0 : reduceError(e.cause)
  };
}
__name(reduceError, "reduceError");
var jsonError = /* @__PURE__ */ __name(async (request, env, _ctx, middlewareCtx) => {
  try {
    return await middlewareCtx.next(request, env);
  } catch (e) {
    const error = reduceError(e);
    return Response.json(error, {
      status: 500,
      headers: { "MF-Experimental-Error-Stack": "true" }
    });
  }
}, "jsonError");
var middleware_miniflare3_json_error_default = jsonError;

// .wrangler/tmp/bundle-vPt2v8/middleware-insertion-facade.js
var __INTERNAL_WRANGLER_MIDDLEWARE__ = [
  middleware_ensure_req_body_drained_default,
  middleware_miniflare3_json_error_default
];
var middleware_insertion_facade_default = gt;

// ../../../../../../usr/lib/node_modules/wrangler/templates/middleware/common.ts
var __facade_middleware__ = [];
function __facade_register__(...args) {
  __facade_middleware__.push(...args.flat());
}
__name(__facade_register__, "__facade_register__");
function __facade_invokeChain__(request, env, ctx, dispatch, middlewareChain) {
  const [head, ...tail] = middlewareChain;
  const middlewareCtx = {
    dispatch,
    next(newRequest, newEnv) {
      return __facade_invokeChain__(newRequest, newEnv, ctx, dispatch, tail);
    }
  };
  return head(request, env, ctx, middlewareCtx);
}
__name(__facade_invokeChain__, "__facade_invokeChain__");
function __facade_invoke__(request, env, ctx, dispatch, finalMiddleware) {
  return __facade_invokeChain__(request, env, ctx, dispatch, [
    ...__facade_middleware__,
    finalMiddleware
  ]);
}
__name(__facade_invoke__, "__facade_invoke__");

// .wrangler/tmp/bundle-vPt2v8/middleware-loader.entry.ts
var __Facade_ScheduledController__ = class ___Facade_ScheduledController__ {
  constructor(scheduledTime, cron, noRetry) {
    this.scheduledTime = scheduledTime;
    this.cron = cron;
    this.#noRetry = noRetry;
  }
  static {
    __name(this, "__Facade_ScheduledController__");
  }
  #noRetry;
  noRetry() {
    if (!(this instanceof ___Facade_ScheduledController__)) {
      throw new TypeError("Illegal invocation");
    }
    this.#noRetry();
  }
};
function wrapExportedHandler(worker) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return worker;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  const fetchDispatcher = /* @__PURE__ */ __name(function(request, env, ctx) {
    if (worker.fetch === void 0) {
      throw new Error("Handler does not export a fetch() function.");
    }
    return worker.fetch(request, env, ctx);
  }, "fetchDispatcher");
  return {
    ...worker,
    fetch(request, env, ctx) {
      const dispatcher = /* @__PURE__ */ __name(function(type, init) {
        if (type === "scheduled" && worker.scheduled !== void 0) {
          const controller = new __Facade_ScheduledController__(
            Date.now(),
            init.cron ?? "",
            () => {
            }
          );
          return worker.scheduled(controller, env, ctx);
        }
      }, "dispatcher");
      return __facade_invoke__(request, env, ctx, dispatcher, fetchDispatcher);
    }
  };
}
__name(wrapExportedHandler, "wrapExportedHandler");
function wrapWorkerEntrypoint(klass) {
  if (__INTERNAL_WRANGLER_MIDDLEWARE__ === void 0 || __INTERNAL_WRANGLER_MIDDLEWARE__.length === 0) {
    return klass;
  }
  for (const middleware of __INTERNAL_WRANGLER_MIDDLEWARE__) {
    __facade_register__(middleware);
  }
  return class extends klass {
    #fetchDispatcher = /* @__PURE__ */ __name((request, env, ctx) => {
      this.env = env;
      this.ctx = ctx;
      if (super.fetch === void 0) {
        throw new Error("Entrypoint class does not define a fetch() function.");
      }
      return super.fetch(request);
    }, "#fetchDispatcher");
    #dispatcher = /* @__PURE__ */ __name((type, init) => {
      if (type === "scheduled" && super.scheduled !== void 0) {
        const controller = new __Facade_ScheduledController__(
          Date.now(),
          init.cron ?? "",
          () => {
          }
        );
        return super.scheduled(controller);
      }
    }, "#dispatcher");
    fetch(request) {
      return __facade_invoke__(
        request,
        this.env,
        this.ctx,
        this.#dispatcher,
        this.#fetchDispatcher
      );
    }
  };
}
__name(wrapWorkerEntrypoint, "wrapWorkerEntrypoint");
var WRAPPED_ENTRY;
if (typeof middleware_insertion_facade_default === "object") {
  WRAPPED_ENTRY = wrapExportedHandler(middleware_insertion_facade_default);
} else if (typeof middleware_insertion_facade_default === "function") {
  WRAPPED_ENTRY = wrapWorkerEntrypoint(middleware_insertion_facade_default);
}
var middleware_loader_entry_default = WRAPPED_ENTRY;
export {
  wt as ContainerStartupOptions,
  dt as IntoUnderlyingByteSource,
  lt as IntoUnderlyingSink,
  pt as IntoUnderlyingSource,
  ht as MinifyConfig,
  yt as R2Range,
  __INTERNAL_WRANGLER_MIDDLEWARE__,
  middleware_loader_entry_default as default
};
//# sourceMappingURL=index.js.map
