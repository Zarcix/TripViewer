from collections import defaultdict

from handlers.file_handler import FileHandler
from handlers.folium_map_handler import FoliumMapHandler

import tkinter
from tkinter import filedialog

renderHandler = FoliumMapHandler()


def folder_picker():
    root = tkinter.Tk()
    root.withdraw()
    file_path = filedialog.askdirectory()
    return file_path


def main():
    # Get Images
    directory = folder_picker()
    file_handler = FileHandler(directory)
    raw_metadata = file_handler.grab_metadata()
    metadata_date_group = file_handler.group_metadata(raw_metadata, group_method="date")

    # Build The Map
    for folder_name, file_metadata in metadata_date_group.items():
        renderHandler.add_feature_group_and_set_context(
            folder_name, show_by_default=True
        )
        renderHandler.add_gps_coords_as_markers(file_metadata)

    renderHandler.finalize_map()

    # Render / Save the Map
    file_path = f"{directory}/map.html"

    renderHandler.render_map(file_path)


if __name__ == "__main__":
    main()
