"""Pin an owned reference child to its actual executable file; no process writes."""
import hashlib
import os
from pathlib import Path

NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
_cache = {}


def native_binary_sha256(pid):
    if type(pid) is not int or pid <= 0:
        raise PermissionError('Requires a positive owned child PID')
    status = Path(f'/proc/{pid}/status').read_text()
    parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
    if int(parent) != os.getpid():
        raise PermissionError('Executable identity requires the observer-owned child')
    executable = Path(f'/proc/{pid}/exe')
    before = executable.stat()
    identity = (pid, before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
    digest = _cache.get(identity)
    if digest is None:
        digest = hashlib.sha256(executable.read_bytes()).hexdigest()
        after = executable.stat()
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns):
            raise RuntimeError('Running executable file changed during identity verification')
        if digest != NATIVE_SHA:
            raise RuntimeError('Owned original executable differs from the pinned base Rebirth image')
        _cache.clear()
        _cache[identity] = digest
    return digest
