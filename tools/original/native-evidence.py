"""Lossless opt-in native evidence storage; never changes clock or inputs."""
import gzip
import hashlib
import json
import importlib.util
from pathlib import Path


def atomic_metadata(path, value):
    temporary = path.with_name(path.name + '.part')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)


def file_digest(path):
    digest = hashlib.sha256()
    size = 0
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
            size += len(chunk)
    return size, digest.hexdigest()


_ZSTD = None


def zstd_module():
    global _ZSTD
    if _ZSTD is None:
        spec = importlib.util.spec_from_file_location(
            'native_journal_zstd', Path(__file__).with_name('native-journal-zstd.py'))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        _ZSTD = module
    return _ZSTD


def make_progress(directory, codec='gzip'):
    if codec == 'gzip':
        return Progress(directory)
    if codec == 'zstd':
        return zstd_module().Progress(directory, atomic_metadata)
    raise ValueError('Unknown Native journal codec')


def progress_path(directory):
    paths = [directory / name for name in ('native-progress.jsonl.gz', 'native-progress.jsonl.zst')]
    present = [path for path in paths if path.exists()]
    if len(present) > 1:
        raise ValueError('Multiple Native progress formats in one capture')
    return present[0] if present else None


def validate_progress_format(path, metadata):
    if path.suffix == '.gz' and (metadata['schema'], metadata['encoding']) == (1, 'gzip-members-jsonl'):
        return
    if path.suffix == '.zst' and (metadata['schema'], metadata['encoding']) == (2, 'zstd-stream-jsonl'):
        if metadata['windowLog'] == 23 and metadata['checksum'] is True:
            return
    raise ValueError('Unknown Native progress format')


def open_progress(path):
    if path.suffix == '.gz':
        return gzip.open(path, 'rb')
    if path.suffix == '.zst':
        return zstd_module().open_closed(path)
    raise ValueError('Unknown Native progress format')


class JournalValues:
    """One final manifest array, streamed from its already committed journal.

    The recorder owns the writer until close. No observation is cached here;
    every decoded value exists only while the JSON encoder consumes that row.
    """
    def __init__(self, progress, kind):
        self.progress = progress
        self.kind = kind

    def __iter__(self):
        progress = self.progress
        if not progress.stream.closed:
            raise ValueError('Close native progress before streaming manifest arrays')
        if file_digest(progress.path) != (progress.archive_bytes, progress.archive_digest.hexdigest()):
            raise ValueError('Native progress has an uncommitted tail or changed archive')
        raw_hash = hashlib.sha256()
        raw_bytes = records = 0
        with open_progress(progress.path) as stream:
            for raw in stream:
                raw_hash.update(raw)
                raw_bytes += len(raw)
                records += 1
                row = json.loads(raw)
                if row['kind'] == self.kind:
                    yield row['value']
        if (records, raw_bytes, raw_hash.hexdigest()) != (
                progress.records, progress.raw_bytes, progress.raw_digest.hexdigest()):
            raise ValueError('Native progress array read is incomplete or changed')


def manifest_chunks(report):
    """Keep the existing pretty-JSON bytes, including lazy top-level arrays."""
    encoder = json.JSONEncoder(indent=2)
    if not isinstance(report, dict) or not any(isinstance(value, JournalValues) for value in report.values()):
        yield from encoder.iterencode(report)
        return
    if not all(isinstance(key, str) for key in report):
        raise ValueError('Native manifest field names must be strings')
    yield '{'
    for index, (key, value) in enumerate(report.items()):
        yield ',\n' if index else '\n'
        yield '  ' + encoder.encode(key) + ': '
        if isinstance(value, JournalValues):
            yield '['
            first = True
            for row in value:
                yield '\n    ' if first else ',\n    '
                first = False
                for text in encoder.iterencode(row):
                    yield text.replace('\n', '\n    ')
            if not first:
                yield '\n  '
            yield ']'
        else:
            for text in encoder.iterencode(value):
                yield text.replace('\n', '\n  ')
    yield '\n}'


def write_manifest(path, report):
    """Decompression is byte-identical to json.dumps(report, indent=2)+'\n'."""
    temporary = path.with_name(path.name + '.part')
    digest = hashlib.sha256()
    size = 0
    with temporary.open('xb') as output:
        with gzip.GzipFile(filename='', fileobj=output, mode='wb', compresslevel=1, mtime=0) as archive:
            pending = []
            buffered = 0
            for text in manifest_chunks(report):
                chunk = text.encode('utf8')
                digest.update(chunk)
                size += len(chunk)
                pending.append(chunk)
                buffered += len(chunk)
                if buffered >= 64 * 1024:
                    archive.write(b''.join(pending))
                    pending.clear()
                    buffered = 0
            if pending:
                archive.write(b''.join(pending))
            archive.write(b'\n')
            digest.update(b'\n')
            size += 1
    temporary.replace(path)
    archive_bytes, archive_sha = file_digest(path)
    evidence = {'schema': 1, 'status': 'closed', 'encoding': 'gzip', 'path': str(path),
                'rawBytes': size, 'rawSha256': digest.hexdigest(),
                'archiveBytes': archive_bytes, 'archiveSha256': archive_sha,
                'scope': 'Complete native manifest, identical decompressed JSON bytes; no observation truncation'}
    atomic_metadata(path.with_name(path.name + '.meta.json'), evidence)
    return evidence


class Progress:
    """Each committed JSONL record is a complete, independently closed gzip member.

    An interrupted write may leave an uncommitted tail. The atomic metadata
    pins the exact committed byte prefix; preserve the entire file on failure.
    """
    def __init__(self, directory):
        self.path = directory / 'native-progress.jsonl.gz'
        self.stream = self.path.open('xb')
        self.raw_digest = hashlib.sha256()
        self.archive_digest = hashlib.sha256()
        self.raw_bytes = self.archive_bytes = self.records = 0
        self.metadata('running')

    def metadata(self, status):
        atomic_metadata(self.path.with_name(self.path.name + '.meta.json'), {
            'schema': 1, 'status': status, 'path': str(self.path), 'encoding': 'gzip-members-jsonl',
            'records': self.records, 'rawBytes': self.raw_bytes, 'rawSha256': self.raw_digest.hexdigest(),
            'committedArchiveBytes': self.archive_bytes, 'archiveSha256': self.archive_digest.hexdigest(),
            'scope': 'Actual completed native progress only; not a completed manifest or passing cohort'})

    def append(self, kind, value):
        raw = (json.dumps({'kind': kind, 'value': value}, separators=(',', ':'), allow_nan=False) + '\n').encode('utf8')
        member = gzip.compress(raw, compresslevel=1, mtime=0)
        self.stream.write(member)
        self.stream.flush()
        self.raw_digest.update(raw)
        self.archive_digest.update(member)
        self.raw_bytes += len(raw)
        self.archive_bytes += len(member)
        self.records += 1
        self.metadata('running')

    def close(self):
        self.stream.close()
        self.metadata('closed')


def read_manifest(path):
    if path.suffix == '.gz':
        metadata = path.with_name(path.name + '.meta.json')
        progress = progress_path(path.parent)
        if progress is not None and progress.suffix == '.zst' and not metadata.exists():
            raise ValueError('Zstd Native evidence requires complete manifest metadata')
        if metadata.exists() and progress is not None:
            manifest_meta = json.loads(metadata.read_text())
            progress_meta = json.loads(progress.with_name(progress.name + '.meta.json').read_text())
            if (manifest_meta['schema'], manifest_meta['encoding']) != (1, 'gzip'):
                raise ValueError('Unknown native evidence format')
            validate_progress_format(progress, progress_meta)
            if manifest_meta['status'] != 'closed' or progress_meta['status'] != 'closed':
                raise ValueError('Native evidence is not closed')
            if file_digest(path) != (manifest_meta['archiveBytes'], manifest_meta['archiveSha256']):
                raise ValueError('Native manifest archive hash/bytes changed')
            if file_digest(progress) != (progress_meta['committedArchiveBytes'], progress_meta['archiveSha256']):
                raise ValueError('Native progress has an uncommitted tail or changed hash/bytes')
            raw_hash = hashlib.sha256()
            raw_bytes = records = 0
            report = None
            finished = False
            with open_progress(progress) as stream:
                for raw in stream:
                    raw_hash.update(raw)
                    raw_bytes += len(raw)
                    records += 1
                    row = json.loads(raw)
                    kind, value = row['kind'], row['value']
                    if finished:
                        raise ValueError('Native progress follows its finish record')
                    if kind == 'start' and report is None and records == 1:
                        report = value
                    elif report is None:
                        raise ValueError('Native progress has no initial report')
                    elif kind == 'acknowledgement':
                        report['presentations'].append(value)
                    elif kind == 'advanceComplete':
                        if report['presentations'][-1]['presentedFrame'] != value['presentedFrame']:
                            raise ValueError('Native progress advance does not match its acknowledgement')
                        report['presentations'][-1] = value
                    elif kind == 'field' and value['name'] in ('bootstrapPresentedFrames', 'declaredOptionsKept', 'initialState'):
                        report[value['name']] = value['value']
                    elif kind in ('debugObservation', 'input', 'capture', 'audioHold', 'focusHandoff'):
                        field = {'debugObservation': 'debugObservations', 'input': 'inputs', 'capture': 'frames',
                                 'audioHold': 'hostAudioHolds', 'focusHandoff': 'focusHandoffs'}[kind]
                        report.setdefault(field, []).append(value)
                    elif kind == 'finish':
                        report.update(value)
                        finished = True
                    else:
                        raise ValueError('Unknown native progress record: ' + kind)
            if not finished or (records, raw_bytes, raw_hash.hexdigest()) != (progress_meta['records'], progress_meta['rawBytes'], progress_meta['rawSha256']):
                raise ValueError('Native progress is incomplete or changed')
            # Verify both the stored complete JSON and its independently
            # reconstructed journal, without one giant JSON string allocation.
            digest = hashlib.sha256()
            size = 0
            for text in json.JSONEncoder(indent=2).iterencode(report):
                chunk = text.encode('utf8')
                digest.update(chunk)
                size += len(chunk)
            digest.update(b'\n')
            if (size + 1, digest.hexdigest()) != (manifest_meta['rawBytes'], manifest_meta['rawSha256']):
                raise ValueError('Native journal does not reconstruct the complete manifest bytes')
            digest = hashlib.sha256()
            size = 0
            with gzip.open(path, 'rb') as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
                    size += len(chunk)
            if (size, digest.hexdigest()) != (manifest_meta['rawBytes'], manifest_meta['rawSha256']):
                raise ValueError('Native manifest decompression hash/bytes changed')
            return report
        with gzip.open(path, 'rt', encoding='utf8') as stream:
            return json.load(stream)
    return json.loads(path.read_text())
