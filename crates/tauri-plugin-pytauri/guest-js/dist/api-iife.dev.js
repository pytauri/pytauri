(function () {
    'use strict';

    /******************************************************************************
    Copyright (c) Microsoft Corporation.

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted.

    THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
    REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
    AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
    INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
    LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
    OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
    PERFORMANCE OF THIS SOFTWARE.
    ***************************************************************************** */
    /* global Reflect, Promise, SuppressedError, Symbol, Iterator */


    typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
        var e = new Error(message);
        return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
    };

    /**
     * Sends a message to the backend.
     * @example
     * ```typescript
     * import { invoke } from '@tauri-apps/api/core';
     * await invoke('login', { user: 'tauri', password: 'poiwe3h4r5ip3yrhtew9ty' });
     * ```
     *
     * @param cmd The command name.
     * @param args The optional arguments to pass to the command.
     * @param options The request options.
     * @return A promise resolving or rejecting to the backend response.
     *
     * @since 1.0.0
     */
    async function invoke(cmd, args = {}, options) {
        return window.__TAURI_INTERNALS__.invoke(cmd, args, options);
    }

    const PY_INVOKE_TAURI_CMD = "plugin:pytauri|pyfunc";
    const PY_INVOKE_HEADER = "pyfunc";
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
    async function pyInvoke(funcName, body, options) {
        let bodyEncoded;
        if (body === undefined || (body instanceof ArrayBuffer) || (body instanceof Uint8Array)) {
            bodyEncoded = body;
        }
        else {
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

    var pytauri = /*#__PURE__*/Object.freeze({
        __proto__: null,
        pyInvoke: pyInvoke
    });

    if ("__TAURI__" in window) {
        Object.defineProperty(window.__TAURI__, "pytauri", { value: pytauri });
    }

})();
