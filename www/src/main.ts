import "./style.css";
import initExplainer, { explain } from "c-explainer-wasm";

const input = document.getElementById("input")! as HTMLTextAreaElement;
const output = document.getElementById("output")!;
const versionSpan = document.getElementById("version")!;

const defaultInitialCode = "const char *foo(int bar)";
const errorColorClass = "text-red-400";

function showOutput(text: string) {
    output.textContent = text;
    output.classList.remove(errorColorClass);
}

function showError(text: string) {
    output.textContent = text;
    output.classList.add(errorColorClass);
}

// Set the version in the footer
versionSpan.textContent = `v${PKG_VERSION}`;

// Set the initial declaration based on the URL parameter or default value
const url = new URL(window.location.toString());
input.value = url.searchParams.get("code") ?? defaultInitialCode;

output.textContent = "Loading WASM module...";
initExplainer()
    .then(() => {
        // Enable the input field once the WASM module is loaded
        input.disabled = false;
        // Set the initial output based on the initial code
        output.textContent = explain(input.value);
        // Add an event listener to update the output when the input changes
        input.addEventListener("input", () => {
            url.searchParams.set("code", input.value);
            window.history.replaceState(null, "", url.toString());
            try {
                showOutput(explain(input.value));
            } catch (err) {
                let errors = err as string[];
                showError(errors.join("\n"));
            }
        });
    })
    .catch((err: unknown) => {
        showError(`Error initializing WASM module: ${err}`);
        console.error("Error initializing WASM module:", err);
    });
