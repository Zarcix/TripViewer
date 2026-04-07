// Helper to manage storage safely
const storage = {
    set: (key, value) => {
        if (value) {
            localStorage.setItem(key, value);
            // Sets a cookie that expires in 7 days, accessible by all paths
            document.cookie = `${key}=${encodeURIComponent(value)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Strict`;
        } else {
            localStorage.removeItem(key);
            // Expire the cookie immediately to delete it
            document.cookie = `${key}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; SameSite=Strict`;
        }
    },
    get: (key) => {
        // Try LocalStorage first (faster)
        const localVal = localStorage.getItem(key);
        if (localVal) return localVal;

        // Fallback: Parse Cookies
        const name = key + "=";
        const decodedCookie = decodeURIComponent(document.cookie);
        const ca = decodedCookie.split(';');
        for (let i = 0; i < ca.length; i++) {
            let c = ca[i].trim();
            if (c.indexOf(name) === 0) {
                return c.substring(name.length, c.length);
            }
        }
        return null;
    }
};

export function getServer() {
    return storage.get("server_ip");
}

export function getToken() {
    return storage.get("auth");
}

export function setServer(val) {
    storage.set("server_ip", val);
}

export function setToken(val) {
    storage.set("auth", val);
}
