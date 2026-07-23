import subprocess
import sys
import time
import signal

DEV_PORTS = [3000, 4000, 4200, 5173, 8000, 8080, 8888, 9000, 1420]
processes = []

for port in DEV_PORTS:
    p = subprocess.Popen(
        [sys.executable, "-m", "http.server", str(port), "--bind", "0.0.0.0"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    processes.append(p)

def cleanup(signum, frame):
    for p in processes:
        p.terminate()
    sys.exit(0)

signal.signal(signal.SIGTERM, cleanup)
signal.signal(signal.SIGINT, cleanup)

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    cleanup(None, None)
