from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor
from datetime import datetime, timezone
from dataclasses import dataclass
from os import scandir, walk
from os.path import join

import os
import exiftool


@dataclass
class Metadata:
    ParsedDate: datetime
    GPS: list[float, float]
    Path: str


class FileHandler:
    def __init__(self, directory: str):
        self.parent_directory: str = directory
        self.files: list[str] = []

        for dirpath, _, filenames in walk(directory):
            for fname in filenames:
                self.files.append(join(dirpath, fname))

    def _parse_metadata_dates(self, metadata):
        date_time: str = metadata.get("EXIF:DateTimeOriginal") or metadata.get(
            "QuickTime:CreateDate"
        )

        date_time_offset: str = metadata.get("EXIF:OffsetTimeDigitized") or "+00:00"
        if not date_time:
            metadata["ParsedDate"] = None
            return metadata

        combined_date_time = (
            f"{date_time.replace("T", " ").replace("Z", "")}{date_time_offset}"
        )
        try:
            local_dt = datetime.strptime(combined_date_time, "%Y:%m:%d %H:%M:%S%z")
            utc_dt = local_dt.astimezone(timezone.utc)
            metadata["ParsedDate"] = utc_dt
        except Exception:
            metadata["ParsedDate"] = None

        return metadata

    def grab_metadata(self) -> list[Metadata]:
        with exiftool.ExifToolHelper() as et:
            metadatas = et.get_metadata(
                self.files,
                [
                    # Original File
                    "-SourceFile",
                    # GPS Data
                    "-Composite:GPSLatitude",
                    "-Composite:GPSLongitude",
                    "-XMP:GPSLatitude",
                    "-XMP:GPSLongitude",
                    # Date Data
                    "-EXIF:DateTimeOriginal",
                    "-QuickTime:CreateDate",
                    "-EXIF:OffsetTimeDigitized",
                ],
            )

        with ProcessPoolExecutor(max_workers=os.cpu_count()) as executor:
            metadatas = list(executor.map(self._parse_metadata_dates, metadatas))

        metadata_list = [
            Metadata(
                ParsedDate=m.get("ParsedDate"),
                GPS=[
                    float(
                        m.get("Composite:GPSLatitude", m.get("XMP:GPSLatitude", 0.0))
                    ),
                    float(
                        m.get("Composite:GPSLongitude", m.get("XMP:GPSLongitude", 0.0))
                    ),
                ],
                Path=os.path.relpath(m.get("SourceFile"), self.parent_directory),
            )
            for m in metadatas
        ]

        return metadata_list

    def _sort_metadata_by_date(self, metadata_list: list[Metadata]):
        return sorted(metadata_list, key=lambda m: m.ParsedDate)

    def _group_metadata_by_date(self, metadata_list: list[Metadata]):
        metadata_group = defaultdict(list)
        for metadata in metadata_list:
            if metadata.ParsedDate is None:
                continue
            day = metadata.ParsedDate.date().isoformat()
            metadata_group[day].append(metadata)

        with ProcessPoolExecutor(max_workers=os.cpu_count()) as executor:
            metadata_sorted = executor.map(
                self._sort_metadata_by_date, metadata_group.values()
            )

        metadata_group = dict(sorted(zip(metadata_group.keys(), metadata_sorted)))
        return metadata_group

    def group_metadata(self, metadata_list, group_method):
        match group_method:
            case "date":
                return self._group_metadata_by_date(metadata_list)
            case _:
                return None
