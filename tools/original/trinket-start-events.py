"""Optional leader-thread Challenge trinket startup tap; never a source of production inputs.

The sibling helper exclusively owns ptrace waits. The clock controller remains
its original child's real parent and may continue using Popen.poll normally.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import select
import subprocess
import time


def pin(path):
    path = Path(path)
    before = path.stat()
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    digest = digest.hexdigest()
    after = path.stat()
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns')
    if any(getattr(before, name) != getattr(after, name) for name in fields):
        raise RuntimeError('Lifecycle source/evidence changed while reading')
    return {'sha256': digest, 'bytes': before.st_size, 'mode': before.st_mode & 0o777,
            'device': before.st_dev, 'inode': before.st_ino, 'mtimeNs': before.st_mtime_ns}


class Tap:
    def __init__(self, pid, directory, *, timeout):
        self.process = None
        self.metadata = {'schema': 1, 'status': 'starting', 'scope': 'leader-thread actual Give41/Remove41 entry and return; no other-thread completeness or whole-game acceptance',
                         'actorMemoryWritten': False, 'instructionsWritten': False,
                         'cpuDebugRegistersTemporarilyWritten': True}
        self.directory = Path(directory)
        self.directory.mkdir(mode=0o700, exist_ok=False)
        self.timeout = timeout
        self.buffer = b''
        self.pins = {}
        self.log = None
        try:
            here = Path(__file__).resolve().parent
            identity_path = here / 'native-identity.py'
            self.pins = {str(path): pin(path) for path in (Path(__file__), here / 'trinket-start-events.c', identity_path)}
            spec = importlib.util.spec_from_file_location('lifecycle_native_identity', identity_path)
            identity = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(identity)
            # Existing identity owner verifies the canonical file AND direct PPid.
            digest = identity.native_binary_sha256(pid)
            if os.environ.get('ISAAC_ORIGINAL_SHA') != digest:
                raise RuntimeError('Lifecycle tap requires the driver-verified ISAAC_ORIGINAL_SHA')
            executable = Path(f'/proc/{pid}/exe')
            binary = executable.stat()
            data = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
            ticks = int(data[19])
            self.metadata.update({'nativeBinarySha256': digest, 'target': {'pid': pid, 'parentPid': os.getpid(), 'startTicks': ticks,
                'executableDevice': binary.st_dev, 'executableInode': binary.st_ino},
                'sourceBefore': self.pins, 'threadIdsAtAttach': sorted(int(path.name) for path in Path(f'/proc/{pid}/task').iterdir())})
            helper = self.directory / 'trinket-start-tap'
            command = ['cc', '-std=gnu11', '-O2', '-g0', '-Wall', '-Wextra', '-Werror',
                       str(here / 'trinket-start-events.c'), '-o', str(helper)]
            with (self.directory / 'compile.log').open('xb') as log:
                compilation = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
            self.metadata['compile'] = {'command': command, 'actualExitCode': compilation.returncode}
            compilation.check_returncode()
            self.metadata['helper'] = pin(helper)
            self.log = (self.directory / 'tracer.stderr').open('xb')
            self.process = subprocess.Popen([str(helper), str(pid), str(ticks), str(binary.st_dev), str(binary.st_ino),
                str(self.directory / 'events.jsonl'), str(86400)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=self.log, start_new_session=True)
            self.metadata['tracerPid'] = self.process.pid
            self._expect('ARMED')
            self.metadata['status'] = 'armed'
            self._save()
        except BaseException as error:
            self.metadata['error'] = repr(error)
            try:
                self.close()
            except Exception as cleanup:
                self.metadata['cleanupError'] = repr(cleanup)
                self._save()
            raise

    def _save(self):
        temporary = self.directory / 'report.json.part'
        with temporary.open('w') as stream:
            json.dump(self.metadata, stream, indent=2, allow_nan=False)
            stream.write('\n'); stream.flush(); os.fsync(stream.fileno())
        temporary.replace(self.directory / 'report.json')

    def _expect(self, expected):
        deadline = time.monotonic() + self.timeout
        while b'\n' not in self.buffer:
            if time.monotonic() >= deadline:
                raise TimeoutError('Lifecycle tracer response timeout')
            if select.select([self.process.stdout], [], [], min(.1, max(0, deadline - time.monotonic())))[0]:
                chunk = os.read(self.process.stdout.fileno(), 4096)
                if not chunk:
                    raise RuntimeError('Lifecycle tracer closed its response channel')
                self.buffer += chunk
        line, self.buffer = self.buffer.split(b'\n', 1)
        if line.decode('ascii') != expected:
            raise RuntimeError('Lifecycle tracer response: ' + repr(line))

    def gate(self, acknowledgement):
        gate = acknowledgement['presentedFrame']
        ns = acknowledgement['virtualElapsedNs']
        if type(gate) is not int or type(ns) is not int or gate < 0 or ns < 0:
            raise ValueError('Lifecycle checkpoint requires actual gate frame and virtual elapsed ns')
        self.process.stdin.write(f'GATE {gate} {ns}\n'.encode('ascii'))
        self.process.stdin.flush()
        self._expect('GATE')

    def close(self):
        if self.metadata.get('status') in ('pass', 'fail'):
            return self.metadata
        errors = []
        if self.process is not None:
            try:
                if self.process.poll() is None:
                    self.process.stdin.write(b'DETACH\n'); self.process.stdin.flush()
                    self._expect('CLOSED')
            except Exception as error:
                errors.append(repr(error))
            try:
                self.metadata['actualTracerExit'] = self.process.wait(timeout=self.timeout)
            except subprocess.TimeoutExpired:
                # C handles SIGTERM by restoring and detaching. If it cannot,
                # EXITKILL prevents an armed reference child from surviving it.
                self.process.terminate()
                try:
                    self.metadata['actualTracerExit'] = self.process.wait(timeout=6)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.metadata['actualTracerExit'] = self.process.wait()
                errors.append('Lifecycle tracer required forced cleanup')
            for stream in (self.process.stdin, self.process.stdout):
                stream.close()
        if self.log is not None:
            self.log.close()
        events = self.directory / 'events.jsonl'
        if events.exists():
            try:
                self.metadata['journal'] = pin(events)
                count = 0; finish = None; event_count = 0; gate_count = 0
                with events.open() as stream:
                    for count, line in enumerate(stream, 1):
                        row = json.loads(line)
                        if row['record'] != count or finish is not None:
                            raise ValueError('Lifecycle record order/finish changed')
                        if row['kind'] == 'event':
                            event_count += 1
                            if row['eventId'] != event_count:
                                raise ValueError('Lifecycle event IDs changed')
                        elif row['kind'] == 'blocked-gate':
                            gate_count += 1
                        elif row['kind'] == 'finish':
                            finish = row
                if not finish or finish['status'] != 'pass' or not finish['detached'] or not finish['debugRegistersRestoredExactly']:
                    raise ValueError('Lifecycle journal lacks successful restore/detach closure')
                if finish['events'] != event_count or finish['gates'] != gate_count or pin(events) != self.metadata['journal']:
                    raise ValueError('Lifecycle journal counts or bytes changed')
                self.metadata['finish'] = finish
                self.metadata['journal'] = pin(events)
            except Exception as error:
                errors.append(repr(error))
        else:
            errors.append('Lifecycle journal missing')
        try:
            self.metadata['sourceEnd'] = {path: pin(path) for path in self.pins}
        except Exception as error:
            self.metadata['sourceEnd'] = None
            errors.append(repr(error))
        self.metadata['sourceBeforeEqualsEnd'] = self.metadata['sourceEnd'] == self.pins
        if self.metadata.get('actualTracerExit') != 0 or not self.metadata['sourceBeforeEqualsEnd'] or self.metadata.get('error'):
            errors.append('Lifecycle source, startup or actual child exit failed')
        self.metadata.update({'status': 'fail' if errors else 'pass', 'errors': errors})
        self._save()
        if errors:
            raise RuntimeError('Optional lifecycle observer failed: ' + repr(errors))
        return self.metadata
