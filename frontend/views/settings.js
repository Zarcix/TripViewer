import { getServer, getToken, setServer, setToken } from "../localstorage.js";

export function initSettings() {
    const serverInput = document.getElementById("server-ip");
    const tokenInput = document.getElementById("bearer-token");

    serverInput.value = getServer();
    tokenInput.value = getToken();

    document.getElementById("save-settings").onclick = () => {
        setServer(serverInput.value);
        setToken(tokenInput.value);
        alert("Saved.");
    };
}