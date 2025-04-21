import { invoke, Channel as TauriChannel, InvokeOptions } from "@tauri-apps/api/core";

const PY_INVOKE_TAURI_CMD = "plugin:pytauri|pyfunc";
const PY_INVOKE_HEADER = "pyfunc";

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

// we dont support `number[]`, because it will make type checking hard in `pyInvoke` function
type RawHandlerBodyType = ArrayBuffer | Uint8Array;
type RawHandlerReturnType = ArrayBuffer;

/**
 * Invokes a Python function through the Tauri IPC mechanism.
 *
 * @param funcName - The name of the Python function to invoke.
 * @param body - The body to send to the Python function.
 * @param options - See {@link invoke} for more details.
 *
 *     ### NOTE
 *
 *     The following headers are reserved and you should not set them in the options:
 *         - `pyfunc`
 *         - `__PYTAURI*`
 *         - `PyTauri*`
 * @returns A promise resolving or rejecting to the backend response.
 */
export async function rawPyInvoke(
    funcName: string,
    body?: RawHandlerBodyType,
    options?: InvokeOptions
): Promise<RawHandlerReturnType> {
    const headers = new Headers(options?.headers);
    // We silently override it without throwing an exception, because Tauri does the same:
    // ref: <https://github.com/tauri-apps/tauri/pull/13227#discussion_r2041442439>
    headers.set(PY_INVOKE_HEADER, funcName);

    const invokePromise = invoke(PY_INVOKE_TAURI_CMD, body, {
        ...options,
        headers,
    });

    if (process.env.NODE_ENV === "development") {
        // development-time type checking to make sure pytauri ipc implementation is correct
        return await invokePromise.then((response) => {
            if (!(response instanceof ArrayBuffer)) {
                throw new Error(
                    "response is not ArrayBuffer. This is not your fault, \
it's a bug for pytauri, please report this issue."
                );
            }
            return response;
        });
    } else {
        return (await invokePromise) as RawHandlerReturnType;
    }
}

/**
 * Invokes a Python function through the Tauri IPC mechanism.
 *
 * This is wrapper around `rawPyInvoke` that handles JSON serialization and deserialization.
 *
 * @template T - The expected return type of the Python function.
 * @param funcName - The name of the Python function to invoke.
 * @param body - The body to send to the Python function. It will be JSON serialized.
 * @param options - See {@link invoke} for more details.
 *     NOTE: `"pyfunc"` header is reserved for pytauri, so you should not set it in the options.
 * @returns A promise resolving or rejecting to the backend response. It will be JSON deserialized.
 * If you dont want JSON deserialization, use `rawPyInvoke` instead.
 */
export async function pyInvoke<T>(
    funcName: string,
    body?: RawHandlerBodyType | any,
    options?: InvokeOptions
): Promise<T> {
    let bodyEncoded: RawHandlerBodyType;

    if (!(body instanceof ArrayBuffer) && !(body instanceof Uint8Array)) {
        const bodyJson = JSON.stringify(body);
        bodyEncoded = textEncoder.encode(bodyJson);
    } else {
        bodyEncoded = body;
    }

    const resp = await rawPyInvoke(funcName, bodyEncoded, options);

    const respJson = textDecoder.decode(resp);
    return JSON.parse(respJson);
}

type RawChannelMsg = ArrayBuffer;

/**
 * This class is a subclass of {@link TauriChannel}.
 * For the {@link TauriChannel} used by `pytauri`, it always transmits {@link ArrayBuffer}.
 * Therefore, this class adds the {@link addJsonListener} method to help deserialize messages.
 *
 * If you dont need that, you can use {@link TauriChannel} directly.
 *
 * @template T - The expected return type from Python.
 */
export class Channel<T = unknown> extends TauriChannel<RawChannelMsg> {
    /**
     * Equivalent to {@link TauriChannel.onmessage}, but it JSON deserializes the message as object.
     */
    addJsonListener(handler: (response: T) => void) {
        this.onmessage = (bytes) => {
            const msgJson = textDecoder.decode(bytes);
            const response: T = JSON.parse(msgJson);
            handler(response);
        };
    }
}
