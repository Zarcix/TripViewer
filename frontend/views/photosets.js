import { get_photosets, create_photoset, update_photoset, put_photoset, delete_photoset } from "../api.js";

let historyStack = [];
let currentDir = "/"; // root

function pushToHistory(folderName) {
    historyStack.push(folderName);
    updateCurrentDir();
}

function popFromHistory() {
    if (historyStack.length > 0) {
        historyStack.pop();
        updateCurrentDir();
    }
}

// Rebuild currentDir from historyStack
function updateCurrentDir() {
    if (historyStack.length === 0) {
        currentDir = "/";
    } else {
        currentDir = "/" + historyStack.join("/");
    }

    let photosetPath = document.getElementById("photosetPath");
    if (photosetPath) {
        photosetPath.value = currentDir;
    }
}

export async function initPhotoSets() {
    // Path Input
    let photosetPath = document.getElementById("photosetPath");
    photosetPath.value = currentDir;
    photosetPath.addEventListener("keypress", function(event) {
        if (event.key === "Enter") {
            event.preventDefault();

            // Normalize input: remove leading/trailing slashes and split
            const path = photosetPath.value.trim();
            const parts = path.split("/").filter(Boolean); // remove empty parts

            // Reset history stack to reflect the input path
            historyStack = [...parts];

            updateCurrentDir();

            // Load the new photoset
            loadPhotoset();
        }
    });

    // Back Button
    let backButton = document.getElementById("backBtn");
    backButton.onclick = () => navBack();

    // Reload Button
    let reloadButton = document.getElementById("reloadBtn");
    reloadButton.onclick = async () => await loadPhotoset();

    // Create Button
    let createButton = document.getElementById("createBtn")
    createButton.onclick = async () => await createPhotoset();

    await loadPhotoset();
}

async function openPhotoset(name) {
    pushToHistory(name);

    // Load the new photoset
    await loadPhotoset();
}

async function createPhotoset() {
    let name = document.getElementById("createName").value;

    const createDir = currentDir + "/" + name;
    console.log("Creating on " + createDir);
    let create_res = await create_photoset(createDir);
    await loadPhotoset();
}

async function deletePhotoset(name) {
    const deleteDir = "/" + historyStack.join("/") + "/" + name;
    let delete_res = await delete_photoset(deleteDir);
    if (delete_res.status == 409) {
        alert("Photoset Not Empty. Photoset Not Deleted");
    }
    await loadPhotoset();
}

async function loadPhotoset() {
    let photosets = await get_photosets(currentDir);
    let entries = photosets.entries;

    let photosetList = document.getElementById("photosetList");
    photosetList.innerHTML = ""
    entries.forEach(entry => {
        const entryDiv = document.createElement('div');
        entryDiv.classList.add('photoset-entry');

        // Name on first line
        const nameSpan = document.createElement('span');
        nameSpan.textContent = entry.name;
        nameSpan.classList.add('photoset-name');
        entryDiv.appendChild(nameSpan);

        // Buttons on second line
        const buttonDiv = document.createElement('div');
        buttonDiv.classList.add('photoset-buttons');

        if (entry.is_dir) {
            const openBtn = document.createElement('button');
            openBtn.textContent = 'Open';
            openBtn.addEventListener('click', async () => {
                await openPhotoset(entry.name);
            });
            buttonDiv.appendChild(openBtn);
        }

        // Update button (for both files and directories)
        const updateBtn = document.createElement('button');
        updateBtn.textContent = 'Update';
        updateBtn.addEventListener('click', async () => {
            await updatePhotoset(entry.name);
        });
        buttonDiv.appendChild(updateBtn);

        // Delete button (for both files and directories)
        const deleteBtn = document.createElement('button');
        deleteBtn.textContent = 'Delete';
        deleteBtn.addEventListener('click', async () => {
            await deletePhotoset(entry.name);
        });
        buttonDiv.appendChild(deleteBtn);

        entryDiv.appendChild(buttonDiv);
        photosetList.appendChild(entryDiv);
    });
}

async function navBack() {
    popFromHistory();
    loadPhotoset();
}