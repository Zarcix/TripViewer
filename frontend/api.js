import { getServer, getToken } from "./localstorage.js";

export async function request(method, path, body = null, isForm = false) {
    const headers = {};

    const token = getToken();
    if (token) headers["Bearer"] = token;

    let options = { method, headers };

    if (body) {
        if (isForm) {
            options.body = body;
        } else {
            headers["Content-Type"] = "application/json";
            options.body = JSON.stringify(body);
        }
    }

    const res = await fetch(`${getServer()}${path}`, options);

    if (!res.ok) {
        const text = await res.text();
        throw new Error(`${res.status}: ${text}`);
    }

    return res;
}
