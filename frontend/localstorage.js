export function getServer() {
    return localStorage.getItem("server_ip");
}

export function getToken() {
    return localStorage.getItem("bearer_token");
}

export function setServer(val) {
    localStorage.setItem("server_ip", val);
}

export function setToken(val) {
    localStorage.setItem("bearer_token", val);
}