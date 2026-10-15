"""Lossless opt-in native evidence storage; never changes clock or inputs."""
import gzip
import hashlib
import json


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


def write_manifest(path, report):
    """Decompression is byte-identical to json.dumps(report, indent=2)+'\n'."""
    temporary = path.with_name(path.name + '.part')
    digest = hashlib.sha256()
    size = 0
    with temporary.open('xb') as output:
        with gzip.GzipFile(filename='', fileobj=output, mode='wb', compresslevel=1, mtime=0) as archive:
            pending = []
            buffered = 0
            for text in json.JSONEncoder(indent=2).iterencode(report):
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
        progress = path.parent / 'native-progress.jsonl.gz'
        if metadata.exists() and progress.exists():
            manifest_meta = json.loads(metadata.read_text())
            progress_meta = json.loads(progress.with_name(progress.name + '.meta.json').read_text())
            if (manifest_meta['schema'], manifest_meta['encoding'], progress_meta['schema'], progress_meta['encoding']) != (1, 'gzip', 1, 'gzip-members-jsonl'):
                raise ValueError('Unknown native evidence format')
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
            with gzip.open(progress, 'rb') as stream:
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
