import { invoke, InvokeOptions } from "@tauri-apps/api/core";

const PY_INVOKE_TAURI_CMD = "plugin:pytauri|pyfunc";
const PY_INVOKE_HEADER = "pyfunc";

// Compared to tauri:
// - we support non-Map JSON types (`any`)
// - we dont support `number[]`, because it will make type checking hard in `pyInvoke` function
type RawPyInvokeArgs = ArrayBuffer | Uint8Array;
type PyInvokeArgs = RawPyInvokeArgs | any;

/**
 * Invokes a Python function through the Tauri IPC mechanism.
 *
 * @param funcName - The name of the Python function to invoke.
 * @param body - The body to send to the Python function.
 * @param options - See {@link invoke} for more details.
 * @template T - The expected return type of the Python function.
 *
 *     ### NOTE
 *
 *     The following headers are reserved and you should not set them in the options:
 *         - `pyfunc`
 *         - `__PYTAURI*`
 *         - `PyTauri*`
 * @returns A promise resolving or rejecting to the backend response.
 */
export async function pyInvoke<T>(
    funcName: string,
    body?: PyInvokeArgs,
    options?: InvokeOptions
): Promise<T> {
    let bodyEncoded: RawPyInvokeArgs | undefined;

    if (body === undefined || (body instanceof ArrayBuffer) || (body instanceof Uint8Array)) {
        bodyEncoded = body;
    } else {
        const bodyJson = JSON.stringify(body);
        bodyEncoded = new TextEncoder().encode(bodyJson);
    }

    const headers = new Headers(options?.headers);
    // We silently override it without throwing an exception, because Tauri does the same:
    // ref: <https://github.com/tauri-apps/tauri/pull/13227#discussion_r2041442439>
    headers.set(PY_INVOKE_HEADER, funcName);

    return await invoke(PY_INVOKE_TAURI_CMD, bodyEncoded, {
        ...options,
        headers,
    });
}
