(()=>{"use strict";var e,r,t,n,a={661:(e,r,t)=>{t.a(e,(async(e,n)=>{try{t.d(r,{WQ:()=>i.WQ});var a=t(191),i=t(967),o=e([a]);a=(o.then?(await o)():o)[0],(0,i.lI)(a),n()}catch(e){n(e)}}))},967:(e,r,t)=>{let n;function a(e){n=e}t.d(r,{Qn:()=>d,WQ:()=>s,lI:()=>a}),e=t.hmd(e);let i=new("undefined"==typeof TextDecoder?(0,e.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});i.decode();let o=null;function s(e,r){return n.add(e,r)>>>0}function d(e,r){throw new Error((t=e,a=r,t>>>=0,i.decode((null!==o&&0!==o.byteLength||(o=new Uint8Array(n.memory.buffer)),o).subarray(t,t+a))));var t,a}Object.freeze({Success:0,0:"Success",Failure:1,1:"Failure"}),"undefined"==typeof FinalizationRegistry||new FinalizationRegistry((e=>n.__wbg_annotationproperties_free(e>>>0))),"undefined"==typeof FinalizationRegistry||new FinalizationRegistry((e=>n.__wbg_inputoptions_free(e>>>0))),"undefined"==typeof FinalizationRegistry||new FinalizationRegistry((e=>n.__wbg_summaryimageoptions_free(e>>>0))),"undefined"==typeof FinalizationRegistry||new FinalizationRegistry((e=>n.__wbg_summarytablecell_free(e>>>0))),"undefined"==typeof FinalizationRegistry||new FinalizationRegistry((e=>n.__wbg_summarywriteoptions_free(e>>>0)))},26:(e,r,t)=>{t.a(e,(async(e,r)=>{try{var n=t(661),a=e([n]);n=(a.then?(await a)():a)[0],console.log(4===(0,n.WQ)(2,2)),r()}catch(e){r(e)}}))},191:(e,r,t)=>{var n=t(967);e.exports=t.v(r,e.id,"08f53d1746d59831e94e",{"./matrix_step_action_bg.js":{__wbindgen_throw:n.Qn}})}},i={};function o(e){var r=i[e];if(void 0!==r)return r.exports;var t=i[e]={id:e,loaded:!1,exports:{}};return a[e](t,t.exports,o),t.loaded=!0,t.exports}e="function"==typeof Symbol?Symbol("webpack queues"):"__webpack_queues__",r="function"==typeof Symbol?Symbol("webpack exports"):"__webpack_exports__",t="function"==typeof Symbol?Symbol("webpack error"):"__webpack_error__",n=e=>{e&&e.d<1&&(e.d=1,e.forEach((e=>e.r--)),e.forEach((e=>e.r--?e.r++:e())))},o.a=(a,i,o)=>{var s;o&&((s=[]).d=-1);var d,c,u,f=new Set,l=a.exports,p=new Promise(((e,r)=>{u=r,c=e}));p[r]=l,p[e]=e=>(s&&e(s),f.forEach(e),p.catch((e=>{}))),a.exports=p,i((a=>{var i;d=(a=>a.map((a=>{if(null!==a&&"object"==typeof a){if(a[e])return a;if(a.then){var i=[];i.d=0,a.then((e=>{o[r]=e,n(i)}),(e=>{o[t]=e,n(i)}));var o={};return o[e]=e=>e(i),o}}var s={};return s[e]=e=>{},s[r]=a,s})))(a);var o=()=>d.map((e=>{if(e[t])throw e[t];return e[r]})),c=new Promise((r=>{(i=()=>r(o)).r=0;var t=e=>e!==s&&!f.has(e)&&(f.add(e),e&&!e.d&&(i.r++,e.push(i)));d.map((r=>r[e](t)))}));return i.r?c:o()}),(e=>(e?u(p[t]=e):c(l),n(s)))),s&&s.d<0&&(s.d=0)},o.d=(e,r)=>{for(var t in r)o.o(r,t)&&!o.o(e,t)&&Object.defineProperty(e,t,{enumerable:!0,get:r[t]})},o.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),o.o=(e,r)=>Object.prototype.hasOwnProperty.call(e,r),o.v=(e,r,t,n)=>new Promise((function(e,t){try{var{readFile:n}=require("fs"),{join:a}=require("path");n(a(__dirname,(r+"").replace(/(^[.-]|[^a-zA-Z0-9_-])+/g,"_")+".wasm"),(function(r,n){if(r)return t(r);e({arrayBuffer:()=>n})}))}catch(e){t(e)}})).then((e=>e.arrayBuffer())).then((e=>WebAssembly.instantiate(e,n))).then((r=>Object.assign(e,r.instance.exports))),o.p="",o(26)})();