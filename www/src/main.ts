import "./style.css";
import initExplainer, { explain } from "c-explainer-wasm";

const input = document.getElementById("input")! as HTMLTextAreaElement;
const output = document.getElementById("output")!;

const initialCode = "const char *foo(int bar)";

function showOutput(text: string) {
    output.textContent = text;
    output.classList.remove("text-red-300");
}

function showError(text: string) {
    output.textContent = text;
    output.classList.add("text-red-400");
}

// Set the initial declaration in the input textarea
input.value = initialCode;

output.textContent = "Loading WASM module...";
initExplainer()
    .then(() => {
        // Enable the input field once the WASM module is loaded
        input.disabled = false;
        // Set the initial output based on the initial code
        output.textContent = explain(input.value);
        // Add an event listener to update the output when the input changes
        input.addEventListener("input", () => {
            try {
                showOutput(explain(input.value));
            } catch (err) {
                let errors = err as string[];
                showError(errors.join("\n"));
            }
        });
    })
    .catch((err) => {
        showError(`Error initializing WASM module: ${err}`);
        console.error("Error initializing WASM module:", err);
    });
