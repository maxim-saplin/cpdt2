### Task 8 Review — Linux platform-specific operations

**Verdict: Implemented (meets intent)**

#### What’s complete
- **Device enumeration**: Parses `/proc/mounts`, filters virtual filesystems, de-duplicates devices, and uses `statvfs` for capacity metrics. Augments classification via `/sys/block` (rotational and removable) to set `DeviceType`.
- **Direct I/O**:
  - `create_direct_io_file()` attempts `O_DIRECT | O_SYNC`, falls back to `O_SYNC` on `EINVAL`, sets file length.
  - `open_direct_io_file()` similarly tries `O_DIRECT | O_SYNC` with fallback.
- **App data directory**: Uses `XDG_DATA_HOME` if present; otherwise `~/.local/share/disk-speed-test`. Ensures directory exists.
- **Filesystem sync**: `fsync` on a file path when applicable, then global `sync()` as a catch-all.
- **Error handling**: Both enumeration and IO report informative failures; reasonable fallbacks included.
- **Tests**: Cover app data dir, mounts parsing/filters, direct I/O open/create, sync, helper utilities, and basic integration through the platform.

#### Requirements alignment
- Matches: 3.1, 3.2, 3.3, 7.1, 7.2, 7.3, 7.4.

#### Notes/caveats
- **O_DIRECT constraints**: Requires alignment of buffers, sizes, and offsets to the filesystem/device block size; higher-level IO must honor this. The fallback to `O_SYNC` is appropriate when unsupported.
- **Device classification**: Heuristics may misclassify some devices (e.g., certain mappers/RAID). Adequate for initial version; can be refined by reading additional sysfs attributes.
- **Network/loopback mounts**: Virtual FS filtering is present; some network FS may still appear. Caller/CLI can optionally filter display.

#### Recommendations (non-blocking)
- Expose a helper to read logical/physical block size from `/sys/block/<dev>/queue/*_block_size` to assist callers with proper alignment.
- Consider capturing filesystem type in `StorageDevice` (optional) to guide later test behavior.

Overall, the Linux implementation satisfies Task 8 with sensible fallbacks and tests.
