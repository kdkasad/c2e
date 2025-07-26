import "./style.css";
import initExplainer, { explain } from "c2e";

const input = document.getElementById("input")! as HTMLTextAreaElement;
const output = document.getElementById("output")!;
const versionSpan = document.getElementById("version")!;

const defaultInitialCode = `char *const (*(*foo)(const int *[]))[3]`;
const errorColorClass = "text-red-400";

function showOutput(text: string) {
    output.textContent = text;
    output.classList.remove(errorColorClass);
}

function showError(text: string) {
    output.textContent = text;
    output.classList.add(errorColorClass);
}

// Returns a promise that rejects after a specified time
function rejectAfter(ms: number): Promise<void> {
    return new Promise((_resolve, reject) =>
        setTimeout(
            reject,
            ms,
            new Error(`Timed out after ${ms / 1000} seconds`),
        ),
    );
}

function processInput() {
    url.searchParams.set("code", input.value);
    window.history.replaceState(null, "", url.toString());
    if (input.value.trim() === "") {
        return;
    }
    try {
        showOutput(explain(input.value));
    } catch (err) {
        let errors = err as string[];
        showError(errors.join("\n"));
    }
}

// Set the version in the footer
versionSpan.textContent = `v${PKG_VERSION}`;

// Set the initial declaration based on the URL parameter or default value
const url = new URL(window.location.toString());
const codeFromUrl = url.searchParams.get("code");
input.value = codeFromUrl || defaultInitialCode;

// Load the WASM module with a timeout
const wasmLoadTimeoutMS = 10000; // 10 seconds
showOutput("Loading WASM module...");
Promise.race([initExplainer(), rejectAfter(wasmLoadTimeoutMS)])
    .then(() => {
        // Set the initial output based on the initial code
        processInput();
        // Add an event listener to update the output when the input changes
        input.addEventListener("input", processInput);
    })
    .catch((err: unknown) => {
        showError(
            `Error initializing WASM module: ${err}. ` +
                `Ensure your browser supports WebAssembly, then try again.`,
        );
        console.error("Error initializing WASM module:", err);
    });
