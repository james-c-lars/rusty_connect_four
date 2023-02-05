import * as wasm_funcs from "../../rust-wasm/pkg/rusty_connect_four.js";
import init from "../../rust-wasm/pkg/rusty_connect_four.js";
window.wasm_funcs = wasm_funcs;
window.wasm = await init();