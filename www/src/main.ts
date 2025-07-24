import "./style.css";
import initExplainer, { explain } from "c-explainer-wasm";

const input = document.getElementById("input")! as HTMLTextAreaElement;
const output = document.getElementById("output")!;

const initialCode = "const char *foo(int bar)";

// Set the initial declaration in the input textarea
input.value = initialCode;

output.textContent = "Loading WASM module...";
initExplainer()
    .then(() => {
        output.textContent = explain(input.value);
        input.addEventListener("input", () => {
            try {
                output.textContent = explain(input.value);
            } catch (err) {
                let errors = err as string[];
                output.textContent = "Error(s):\n";
                output.textContent += errors.join("\n");
            }
        });
    })
    .catch((err) => {
        output.textContent = `Error initializing WASM module: ${err}`;
        console.error("Error initializing WASM module:", err);
    });
