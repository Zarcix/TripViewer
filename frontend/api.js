import { getServer, getToken } from "./localstorage.js";

const API_PATH = "/api/photoset"

// export async function request(method, path, body = null, isForm = false) {
//     const headers = {};

//     const token = getToken();
//     if (token) headers["Bearer"] = token;

//     let options = { method, headers };

//     if (body) {
//         if (isForm) {
//             options.body = body;
//         } else {
//             headers["Content-Type"] = "application/json";
//             options.body = JSON.stringify(body);
//         }
//     }

//     const res = await fetch(`${getServer()}${path}`, options);

//     if (!res.ok) {
//         const text = await res.text();
//         throw new Error(`${res.status}: ${text}`);
//     }

//     return res;
// }

export function get_apiPath() {
    const server = getServer();
    return `${server}${API_PATH}`
}

export async function get_photosets(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "GET"
    })

    return res;
}

export async function head_photosets(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "HEAD"
    })

    return res;
}

export async function create_photoset(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "POST"
    })

    return res;
}

export async function update_photoset(path, new_name) {
    const server = getServer();

    // Create a FormData object and append the new_name field
    const formData = new FormData();
    formData.append("new_name", new_name);

    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "PATCH",
        body: formData
    });

    return res;
}

export function put_photoset(path, file, onProgress) {
    const server = getServer();
    const xhr = new XMLHttpRequest();

    const promise = new Promise((resolve, reject) => {
        xhr.open("PUT", `${server}${API_PATH}${path}`);
        xhr.setRequestHeader("Content-Type", file.type || "application/octet-stream");

        xhr.upload.onprogress = (event) => {
            if (event.lengthComputable && onProgress) {
                onProgress(event.loaded, event.total);
            }
        };

        xhr.onload = () => {
            resolve({
                ok: xhr.status >= 200 && xhr.status < 300,
                status: xhr.status
            });
        };

        xhr.onerror = () => reject(new Error("Network error"));
        xhr.onabort = () => reject(new Error("Upload aborted"));

        xhr.send(file);
    });

    return { promise, xhr };
}

export async function delete_photoset(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "DELETE"
    })

    return res;
}
