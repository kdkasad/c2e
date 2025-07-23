import './style.css';
import initExplainer, { explain } from 'c-explainer-wasm';

const output = document.getElementById('output')!;

initExplainer().then(() => {
    output.textContent = explain('int foo(void)');
}).catch((err) => {
    output.textContent = `Error initializing WASM module: ${err}`;
});
