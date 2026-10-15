"""Opt-in Native journal codec: persistent Zstd, strict bounded completion reads.

Requires installed Python zstandard and system libzstd; loaded only for new
Zstd evidence. Legacy gzip captures never import this module.
"""
import ctypes as C
import ctypes.util
import hashlib
import io
import json
import zstandard

WINDOW_LOG = 23
WINDOW_BYTES = 1 << WINDOW_LOG
CHUNK = 65536
MAX_RECORD = 32 * 1024 * 1024


class FlushedZstdTransport:
    """Wrap the old writer's output OR receive its literal canonical JSONL.

    write() accepts one record. The caller's existing flush() commits one
    immediately decodable block. close() finishes the frame. All byte counters
    are explicit transport/input counters, never renamed as each other.
    """
    def __init__(self, output):
        self.output = output
        params = zstandard.ZstdCompressionParameters.from_level(
            1, window_log=WINDOW_LOG, write_checksum=1)
        self.compressor = zstandard.ZstdCompressor(compression_params=params).compressobj()
        self.input_digest = hashlib.sha256()
        self.transport_digest = hashlib.sha256()
        self.input_bytes = self.transport_bytes = self.flushed_records = 0
        self.failed = self.finished = self.pending = False

    @property
    def closed(self):
        return self.output.closed

    def emit(self, value):
        if self.output.write(value) != len(value):
            raise OSError('Short journal transport write')
        self.transport_digest.update(value)
        self.transport_bytes += len(value)

    def write(self, record):
        if self.failed or self.finished or self.pending:
            raise ValueError('Record requires a healthy writer and prior flush')
        if not record or len(record) > MAX_RECORD:
            raise ValueError('Explicit nonempty bounded journal record required')
        try:
            for offset in range(0, len(record), CHUNK):
                self.emit(self.compressor.compress(record[offset:offset + CHUNK]))
            self.input_digest.update(record)
            self.input_bytes += len(record)
            self.pending = True
            return len(record)
        except BaseException:
            self.failed = True
            raise

    def flush(self):
        if self.failed or self.finished or not self.pending:
            raise ValueError('Only a pending complete record may be flushed')
        try:
            self.emit(self.compressor.flush(zstandard.COMPRESSOBJ_FLUSH_BLOCK))
            self.output.flush()
            self.flushed_records += 1
            self.pending = False
        except BaseException:
            self.failed = True
            raise

    def close(self):
        if self.finished:
            return
        if self.failed or self.pending:
            self.output.close()
            raise ValueError('Uncommitted/failed transport cannot be closed successfully')
        try:
            self.emit(self.compressor.flush(zstandard.COMPRESSOBJ_FLUSH_FINISH))
            self.output.flush()
            self.output.close()
            self.finished = True
        except BaseException:
            self.failed = True
            self.output.close()
            raise


class _In(C.Structure):
    _fields_ = [('src', C.c_void_p), ('size', C.c_size_t), ('pos', C.c_size_t)]


class _Out(C.Structure):
    _fields_ = [('dst', C.c_void_p), ('size', C.c_size_t), ('pos', C.c_size_t)]


class StrictZstdReader(io.RawIOBase):
    """One completed Zstd frame, bounded buffers/window, explicit EOF proof.

    Python zstandard.stream_reader alone permits truncated source EOF. This
    reader uses libzstd's actual return0 frame boundary and rejects truncation,
    trailing data, checksum failure and additional frames. Running-prefix
    recovery intentionally uses a separate, explicitly incomplete path.
    """
    def __init__(self, source):
        super().__init__()
        self.source = source
        self.context = None
        library = ctypes.util.find_library('zstd')
        if library is None:
            raise RuntimeError('Zstd journals require installed system libzstd')
        self.lib = C.CDLL(library)
        self.lib.ZSTD_createDCtx.restype = C.c_void_p
        self.lib.ZSTD_freeDCtx.argtypes = [C.c_void_p]
        self.lib.ZSTD_freeDCtx.restype = C.c_size_t
        self.lib.ZSTD_DCtx_setParameter.argtypes = [C.c_void_p, C.c_int, C.c_int]
        self.lib.ZSTD_DCtx_setParameter.restype = C.c_size_t
        self.lib.ZSTD_decompressStream.argtypes = [C.c_void_p, C.POINTER(_Out), C.POINTER(_In)]
        self.lib.ZSTD_decompressStream.restype = C.c_size_t
        self.lib.ZSTD_isError.argtypes = [C.c_size_t]
        self.lib.ZSTD_isError.restype = C.c_uint
        self.lib.ZSTD_getErrorName.argtypes = [C.c_size_t]
        self.lib.ZSTD_getErrorName.restype = C.c_char_p
        self.context = self.lib.ZSTD_createDCtx()
        if not self.context:
            raise MemoryError('Zstd decoder allocation failed')
        try:
            self.check(self.lib.ZSTD_DCtx_setParameter(self.context, 100, WINDOW_LOG))
        except BaseException:
            self.lib.ZSTD_freeDCtx(self.context)
            self.context = None
            raise
        self.input = _In(None, 0, 0)
        self.input_storage = None
        self.source_eof = self.frame_complete = False
        self.header_checked = False

    def check(self, result):
        if self.lib.ZSTD_isError(result):
            raise ValueError(self.lib.ZSTD_getErrorName(result).decode('utf8'))
        return result

    def readable(self):
        return True

    def readinto(self, destination):
        if self.closed:
            raise ValueError('Reader is closed')
        if not len(destination) or self.frame_complete:
            return 0
        while True:
            if self.input.pos == self.input.size and not self.source_eof:
                data = self.source.read(CHUNK)
                self.source_eof = not data
                if not self.header_checked:
                    if data[:6] != bytes.fromhex('28b52ffd0468'):
                        raise ValueError('Unknown Native journal Zstd header/window')
                    self.header_checked = True
                self.input_storage = C.create_string_buffer(data) if data else None
                self.input = _In(C.cast(self.input_storage, C.c_void_p), len(data), 0)
            size = min(len(destination), CHUNK)
            target = (C.c_char * size).from_buffer(destination)
            output = _Out(C.cast(target, C.c_void_p), size, 0)
            result = self.check(self.lib.ZSTD_decompressStream(
                self.context, C.byref(output), C.byref(self.input)))
            if result == 0:
                if self.input.pos != self.input.size or self.source.read(1):
                    raise ValueError('Trailing data or extra frame after complete journal')
                self.frame_complete = True
            if output.pos:
                return output.pos
            if self.frame_complete:
                return 0
            if self.source_eof:
                raise EOFError('Journal Zstd frame is unfinished')

    def close(self):
        if not self.closed:
            try:
                self.source.close()
            finally:
                if self.context:
                    self.lib.ZSTD_freeDCtx(self.context)
                    self.context = None
                super().close()


def open_closed(path):
    """Caller first verifies closed metadata and actual encoded bytes/SHA."""
    return io.BufferedReader(StrictZstdReader(path.open('rb')), buffer_size=CHUNK)


class Progress:
    """New format only; literal JSONL and existing record/flush gates retained."""
    def __init__(self, directory, atomic_metadata):
        self.path = directory / 'native-progress.jsonl.zst'
        self.atomic_metadata = atomic_metadata
        # Resolve codec/decoder requirements before launching an original.
        if ctypes.util.find_library('zstd') is None:
            raise RuntimeError('Zstd journals require installed system libzstd')
        self.stream = FlushedZstdTransport(self.path.open('xb'))
        self.raw_digest = hashlib.sha256()
        self.raw_bytes = self.records = 0
        self.failed = False
        try:
            self.metadata('running')
        except BaseException:
            self.failed = self.stream.failed = True
            self.stream.output.close()
            raise

    @property
    def archive_bytes(self):
        return self.stream.transport_bytes

    @property
    def archive_digest(self):
        return self.stream.transport_digest

    def metadata(self, status):
        self.atomic_metadata(self.path.with_name(self.path.name + '.meta.json'), {
            'schema': 2, 'status': status, 'path': str(self.path), 'encoding': 'zstd-stream-jsonl',
            'windowLog': WINDOW_LOG, 'checksum': True,
            'records': self.records, 'rawBytes': self.raw_bytes, 'rawSha256': self.raw_digest.hexdigest(),
            'committedArchiveBytes': self.archive_bytes, 'archiveSha256': self.archive_digest.hexdigest(),
            'scope': 'Actual completed native progress only; not a completed manifest or passing cohort'})

    def append(self, kind, value):
        if self.failed:
            raise ValueError('Failed native progress cannot accept records')
        raw = (json.dumps({'kind': kind, 'value': value}, separators=(',', ':'), allow_nan=False) + '\n').encode('utf8')
        try:
            self.stream.write(raw)
            self.stream.flush()
            self.raw_digest.update(raw)
            self.raw_bytes += len(raw)
            self.records += 1
            self.metadata('running')
        except BaseException:
            self.failed = self.stream.failed = True
            raise

    def close(self):
        if self.failed:
            self.stream.output.close()
            raise ValueError('Failed Native progress cannot be certified closed')
        try:
            self.stream.close()
            self.metadata('closed')
        except BaseException:
            self.failed = self.stream.failed = True
            self.stream.output.close()
            raise
