import { getServer, getToken, setServer, setToken } from "../localstorage.js";

export function initSettings() {
    const serverInput = document.getElementById("serverIP");
    const tokenInput = document.getElementById("bearerToken");

    serverInput.value = getServer();
    tokenInput.value = getToken();

    document.getElementById("saveSettings").onclick = () => {
        setServer(serverInput.value);
        setToken(tokenInput.value);
        alert("Saved.");
    };
}