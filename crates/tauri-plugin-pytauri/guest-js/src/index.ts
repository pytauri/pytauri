import { invoke } from "@tauri-apps/api/core";

const PY_INVOKE_TAURI_CMD = "plugin:pytauri|pyfunc";
const PY_INVOKE_HEADER = "pyfunc";

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

// we dont support `number[]`, because it will make type checking hard in `pyInvoke` function
type RawHandlerBodyType = ArrayBuffer | Uint8Array;
type RawHandlerReturnType = ArrayBuffer;

export async function rawPyInvoke(
    funcName: string,
    body: RawHandlerBodyType
): Promise<RawHandlerReturnType> {
    const invokePromise = invoke(PY_INVOKE_TAURI_CMD, body, {
        headers: { [PY_INVOKE_HEADER]: funcName },
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

export async function pyInvoke<T>(
    funcName: string,
    body: object | RawHandlerBodyType
): Promise<T> {
    let bodyEncoded: RawHandlerBodyType;

    if (!(body instanceof ArrayBuffer) && !(body instanceof Uint8Array)) {
        const bodyJson = JSON.stringify(body);
        bodyEncoded = textEncoder.encode(bodyJson);
    } else {
        bodyEncoded = body;
    }

    const resp = await rawPyInvoke(funcName, bodyEncoded);

    const respJson = textDecoder.decode(resp);
    return JSON.parse(respJson);
}
