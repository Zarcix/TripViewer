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

export async function get_photosets(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "GET"
    })

    return await res.json()
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

export async function put_photoset(path, file) {
    const server = getServer();

    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "PUT",
        body: file, // file should be a File or Blob object
    });

    return await res.json();
}

export async function delete_photoset(path) {
    const server = getServer();
    const res = await fetch(`${server}${API_PATH}${path}`, {
        method: "DELETE"
    })

    return res;
}
