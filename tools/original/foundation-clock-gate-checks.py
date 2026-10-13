#!/usr/bin/env python3
"""Check the typed public time/presentation ABI without any game dependency."""
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent


def main():
    with tempfile.TemporaryDirectory(prefix='clock-gate-test-') as directory:
        out = Path(directory)
        (out / 'swap-stub.c').write_text('''void glXSwapBuffers(void *display,unsigned long drawable){(void)display;(void)drawable;}
int XNextEvent(void *display,void *event){(void)display;*(int*)event=2;return 0;}
int XCheckTypedEvent(void *display,int type,void *event){(void)display;*(int*)event=type;return 1;}
int XCheckIfEvent(void *display,void *event,int (*predicate)(void *,void *,char *),char *argument){*(int*)event=2;return predicate(display,event,argument);}
unsigned long XLookupKeysym(void *event,int index){(void)event;(void)index;return 42;}
''')
        subprocess.run(['cc', '-Wall', '-Wextra', '-Werror', '-shared', '-fPIC', str(out / 'swap-stub.c'), '-o', str(out / 'libswap-stub.so')], check=True)
        subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared', str(HERE / 'foundation-clock-gate.c'), '-o', str(out / 'gate.so'), '-ldl'], check=True)
        subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', str(HERE / 'foundation-clock-gate-test.c'), '-o', str(out / 'abi-test'), '-L' + str(out), '-lswap-stub', '-pthread'], check=True)
        address = str(out / 'socket')
        env = os.environ.copy()
        env.update({'LD_PRELOAD': str(out / 'gate.so'), 'LD_LIBRARY_PATH': str(out),
                    'ISAAC_CLOCK_GATE_SOCKET': address, 'ISAAC_CLOCK_GATE_STEP_NS': '16667000'})
        proc = subprocess.Popen([str(out / 'abi-test')], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        connection.settimeout(5)
        try:
            deadline = time.monotonic() + 5
            while True:
                try:
                    connection.connect(address)
                    break
                except (FileNotFoundError, ConnectionRefusedError):
                    if proc.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError('ABI test did not open the presentation gate')
                    time.sleep(.01)
            reader = connection.makefile('r')
            reports = [json.loads(reader.readline())]
            assert reports[0]['presentedFrame'] == 0 and reports[0]['virtualElapsedNs'] == 0
            assert [event['down'] for event in reports[0]['consumedX11Events']] == [True, False, True]
            assert all(event['keysym'] == 42 and event['presentedFramePhase'] == 0 for event in reports[0]['consumedX11Events'])
            assert [event['focused'] for event in reports[0]['consumedX11FocusEvents']] == [True, False]
            assert all(event['presentedFramePhase'] == 0 and event['virtualElapsedNs'] == 0 for event in reports[0]['consumedX11FocusEvents'])
            connection.sendall(b'step 1\n')
            reports.append(json.loads(reader.readline()))
            assert reports[-1]['presentedFrame'] == 1 and reports[-1]['virtualElapsedNs'] == 16667000
            for invalid in [b'step 0\n', b'step -1\n', b'step 36001\n', b'step 2 extra\n', b'unknown\n']:
                connection.sendall(invalid)
                assert 'error' in json.loads(reader.readline())
            connection.sendall(b'step 2\n')
            reports.append(json.loads(reader.readline()))
            assert reports[-1]['presentedFrame'] == 3 and reports[-1]['virtualElapsedNs'] == 50001000
            connection.sendall(b'quit\n')
            stdout, stderr = proc.communicate(timeout=5)
            if proc.returncode:
                raise RuntimeError(stderr)
            result = json.loads(stdout)
            assert result['status'] == 'pass' and result['assertions'] >= 30
            print(json.dumps({**result, 'protocolAssertions': 12, 'workerAndCpuClocksReal': True, 'simulationTickClaim': False}))
        finally:
            connection.close()
            if proc.poll() is None:
                proc.kill()
                proc.wait()

if __name__ == '__main__':
    main()
