from fastmcp import FastMCP
import subprocess
import json
import os
import time
import signal
import sys

mcp = FastMCP("friskingdom-debug")

DEBUG_DIR = "/tmp/friskingdom-debug"
CARGO_DIR = "/Users/ember/dev/friskingdom"
_game_process = None


@mcp.tool()
def launch_game() -> dict:
    """Launch FrisKingdom with the debug plugin enabled. Returns status and PID.
    The game will auto-dump state to /tmp/friskingdom-debug/state.json every second
    and screenshots to /tmp/friskingdom-debug/screenshot.png every 2 seconds."""
    global _game_process
    if _game_process and _game_process.poll() is None:
        return {"status": "already_running", "pid": _game_process.pid}

    os.makedirs(DEBUG_DIR, exist_ok=True)
    # Clean old state files
    for f in ["state.json", "screenshot.png", "commands.json"]:
        path = os.path.join(DEBUG_DIR, f)
        if os.path.exists(path):
            os.remove(path)

    _game_process = subprocess.Popen(
        ["cargo", "run", "--features", "debug"],
        cwd=CARGO_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Wait for the game to start and produce first state dump (up to 60s for compilation)
    for _ in range(600):
        if _game_process.poll() is not None:
            stderr = _game_process.stderr.read().decode()[-2000:]
            return {"status": "crashed", "exit_code": _game_process.returncode, "stderr": stderr}
        if os.path.exists(os.path.join(DEBUG_DIR, "state.json")):
            return {"status": "running", "pid": _game_process.pid}
        time.sleep(0.1)

    return {"status": "timeout", "pid": _game_process.pid, "message": "Game started but no state.json appeared within 60s"}


@mcp.tool()
def stop_game() -> dict:
    """Stop the running game process."""
    global _game_process
    if _game_process is None or _game_process.poll() is not None:
        return {"status": "not_running"}

    _game_process.send_signal(signal.SIGTERM)
    try:
        _game_process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        _game_process.kill()
        _game_process.wait()

    _game_process = None
    return {"status": "stopped"}


@mcp.tool()
def game_state() -> dict:
    """Read the latest game state from the debug dump.
    Returns game phase, scores, stall count, disc state, and all player positions."""
    path = os.path.join(DEBUG_DIR, "state.json")
    if not os.path.exists(path):
        return {"error": "No state file found. Is the game running with --features debug?"}
    try:
        with open(path) as f:
            return json.load(f)
    except json.JSONDecodeError as e:
        return {"error": f"Invalid JSON in state file: {e}"}


@mcp.tool()
def screenshot() -> dict:
    """Get the path to the latest screenshot. Use the Read tool to view it as an image.
    Screenshots are auto-captured every 2 seconds, or use send_command('screenshot') to trigger one."""
    path = os.path.join(DEBUG_DIR, "screenshot.png")
    if not os.path.exists(path):
        return {"error": "No screenshot found. Game may not have rendered yet."}
    mtime = os.path.getmtime(path)
    age = time.time() - mtime
    return {
        "path": path,
        "age_seconds": round(age, 1),
        "message": f"Screenshot at {path} (captured {round(age, 1)}s ago). Use Read tool to view it."
    }


@mcp.tool()
def send_command(command: str, data: dict = None) -> dict:
    """Send a command to the running game via the debug plugin.

    Available commands:
    - 'screenshot': Take a screenshot immediately
    - 'pause': Pause the game
    - 'resume': Resume the game
    - 'step': Advance one frame (when paused)
    - 'set_speed': Set game speed (data: {"multiplier": 3.0})
    - 'inject_input': Set game input (data: {"throw_released": true, "throw_power": 0.7, "throw_type": "Backhand", "move_dir": [0.5, 1.0], "sprint": false})
    - 'dump_state': Request immediate state dump
    """
    cmd = {"command": command}
    if data:
        cmd["data"] = data

    path = os.path.join(DEBUG_DIR, "commands.json")
    with open(path, "w") as f:
        json.dump(cmd, f)

    return {"status": "sent", "command": command}


@mcp.tool()
def run_tests(test_name: str = "") -> dict:
    """Run headless integration tests via cargo test.

    Args:
        test_name: Optional specific test name to run (e.g. 'test_phase_transitions'). Empty string runs all tests.
    """
    cmd = ["cargo", "test", "-p", "fk-game", "--", "--nocapture"]
    if test_name:
        cmd.insert(4, test_name)  # Insert before --

    try:
        result = subprocess.run(
            cmd, cwd=CARGO_DIR,
            capture_output=True, text=True, timeout=120
        )
        return {
            "exit_code": result.returncode,
            "passed": result.returncode == 0,
            "stdout": result.stdout[-4000:] if result.stdout else "",
            "stderr": result.stderr[-4000:] if result.stderr else "",
        }
    except subprocess.TimeoutExpired:
        return {"error": "Test timed out after 120 seconds"}


@mcp.tool()
def game_is_running() -> dict:
    """Check if the game process is currently running."""
    global _game_process
    if _game_process is None:
        return {"running": False}

    poll = _game_process.poll()
    if poll is not None:
        return {"running": False, "exit_code": poll}
    return {"running": True, "pid": _game_process.pid}


def main():
    mcp.run()


if __name__ == "__main__":
    main()
