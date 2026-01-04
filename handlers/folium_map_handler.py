from collections import defaultdict
from dataclasses import dataclass
from typing import Sequence
from folium.plugins import BeautifyIcon
from folium import Map, Marker, FeatureGroup, Popup, LayerControl

import colorsys
import random

VIDEO_MIME_TYPES = {
    "mp4": "video/mp4",
    "mov": "video/quicktime",
    "mkv": "video/x-matroska",
    "webm": "video/webm",
    "avi": "video/x-msvideo",
    "flv": "video/x-flv",
    "wmv": "video/x-ms-wmv",
    "m4v": "video/x-m4v",
    "mpeg": "video/mpeg",
}


MEDIA_STYLE = "max-height: 15vw; max-width: 10vw; object-fit: contain;"
POPUP_STYLE = "max-height: 15vw; overflow: scroll"
HEADER_MIN_WIDTH = "15vw"


@dataclass
class FeatureGroupData:
    feature_number: int
    feature_color: str


class FoliumMapHandler:
    def __init__(
        self, starting_location: Sequence[float] | None = [0, 0], zoom: int = 3
    ):
        self.folium_map = Map(
            location=starting_location, min_zoom=zoom, zoom_start=zoom
        )

        self.feature_group_data: dict[FeatureGroup, FeatureGroupData] = {}
        self.feature_group = None
        self.total_feature_count = 0

    def _get_random_feature_color(self):
        h = random.random()
        s = 0.8
        v = 0.9
        r, g, b = [int(x * 255) for x in colorsys.hsv_to_rgb(h, s, v)]
        return f"#{r:02x}{g:02x}{b:02x}"

    def add_feature_group_and_set_context(self, group_name: str, show_by_default=False):
        """
        Creates a new feature group and sets it as the current feature group context.
        This group context will not change until a new feature group is added
        """
        feature_group = FeatureGroup(name=group_name, show=show_by_default).add_to(
            self.folium_map
        )

        self.total_feature_count += 1
        self.feature_group = feature_group

        feature_data = FeatureGroupData(
            feature_number=self.total_feature_count,
            feature_color=self._get_random_feature_color(),
        )
        self.feature_group_data[feature_group] = feature_data

    def set_feature_group_context(self, new_group: FeatureGroup):
        if new_group not in self.feature_group_data.keys():
            raise IndexError("New feature group not found in created feature groups")

        self.feature_group = new_group

    def _make_media_html(self, metadata):
        """Return clickable <img> or <video> HTML depending on file type."""
        path = metadata.Path
        ext = path.lower().rsplit(".", 1)[-1]

        mime = VIDEO_MIME_TYPES.get(ext)
        if mime:  # video
            return f"""
<div>
    <video controls style='{MEDIA_STYLE}'>
        <source src='{path}' type='{mime}'>
    </video>
    <div>
        <a href='{path}' target='_blank' rel='noopener noreferrer'>
            Open video in new tab
        </a>
    </div>
</div>
"""

        # image
        return f"""
<div>
    <a href='{path}' target='_blank' rel='noopener noreferrer'>
        <img src='{path}' style='{MEDIA_STYLE}'>
    </a>
</div>
"""

    def _build_popup_html(self, coords, metadata_list):
        """Build the full popup HTML for all media at a coordinate."""
        html = f"""
<div style='{POPUP_STYLE}'>
    <div style='min-width: {HEADER_MIN_WIDTH};'>
        <h1>Images at this location:</h1>
        <h6>Note that all the medias are collapsable</h6>
    </div>
"""

        for index, metadata in metadata_list:
            media_html = self._make_media_html(metadata)
            # path = metadata.Path

            html += f"""
<details open>
    <summary style='font-size: 1.5em; font-weight: bold'>
        ⬘ Media Number {index}
    </summary>
    {media_html}
</details>
"""

        html += "</div>"
        return html

    def _build_index_range(self, metadata_list):
        # Grab Indexes
        indexes = [m[0] for m in metadata_list]

        ranges = []
        start = prev = indexes[0]

        for n in indexes[1:]:
            if n != prev + 1:
                ranges.append(f"{start}-{prev}" if start != prev else str(start))
                start = n
            prev = n

        ranges.append(f"{start}-{prev}" if start != prev else str(start))

        return ", ".join(ranges)

    def add_gps_coords_as_markers(self, gps_group):
        feature_data = self.feature_group_data[self.feature_group]
        # Group Coordinates based on groups
        coord_groups = defaultdict(list)

        # Populate Coordinates
        for coords, metadata_list in gps_group.items():
            html = self._build_popup_html(coords, metadata_list)
            coord_groups[coords].append(Popup(html, max_width="500%", lazy=True))

            index_range = self._build_index_range(metadata_list)
            coord_groups[coords].append(index_range)

        # Create and Place Markers
        for coord, (popup, index_range) in coord_groups.items():
            icon_number = BeautifyIcon(
                border_color=feature_data.feature_color,
                text_color="#00ABDC",
                number=feature_data.feature_number,
                inner_icon_style="margin-top:0;",
            )
            Marker(coord, popup=popup, icon=icon_number, tooltip=index_range).add_to(
                self.feature_group
            )

    def finalize_map(self):
        LayerControl().add_to(self.folium_map)

    def render_map(self, save_path):
        self.folium_map.save(save_path)

    def preview_map(self):
        self.folium_map.show_in_browser()
