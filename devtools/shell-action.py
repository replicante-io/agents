import argparse
import json
import os
import subprocess
import sys

from argparse import ArgumentParser


### Supported actions ###
def check(args):
    action = json.load(sys.stdin)
    action_id = action['id']

    # Load the latest status, if possible.
    status = None
    status_path = f"/var/run/user/{args.uid}/{action_id}"
    try:
        status = open(status_path, 'r')
        status = json.load(status)
    except IOError:
        status = {
            "status": "failed",
            "error": "status file not found",
        }
    except ValueError:
        status = {
            "status": "failed",
            "error": "unable to decode status file",
        }

    # Notify the agent about the status and clean up.
    json.dump(status, sys.stdout)
    if status['status'] != 'running':
        try:
            os.remove(status_path)
        except Exception:
            # Ignore attemps to remove missing file.
            pass


def run(args):
    action = json.load(sys.stdin)
    action_id = action['id']

    # Create the action status file before returning to the agent.
    # This can lead to endless waiting if the action fails but the script is unable to update
    # the status file with information about the error.
    status_path = f"/var/run/user/{args.uid}/{action_id}"
    status = {"status": "running"}
    json.dump(status, open(status_path, 'w'))

    # In this script we simply fork to release the agent.
    # This is generally not good enough because the agent is still our parent.
    # If the agent exits the action will be killed as well.
    newpid = os.fork()
    if newpid > 0:
        print("Action process forked")
        sys.exit(0)

    # Decouple from parent environment.
    os.chdir("/")
    os.setsid()
    os.umask(0)
    newpid = os.fork()
    if newpid > 0:
        print("Forked the second time")
        sys.exit(0)

    # Redirect standard file descriptors
    sys.stdout.flush()
    sys.stderr.flush()
    si = open('/dev/null', 'r')
    so = open('/dev/null', 'a+')
    se = open('/dev/null', 'a+')
    os.dup2(si.fileno(), sys.stdin.fileno())
    os.dup2(so.fileno(), sys.stdout.fileno())
    os.dup2(se.fileno(), sys.stderr.fileno())

    # Run the action.
    output = subprocess.run(args.command, text=True, capture_output=True)

    # Note that the action completed.
    status = {"status": "finished"}
    if output.returncode != 0:
        status["status"] = "failed"
        status["error"] = output.stderr
    json.dump(status, open(status_path, 'w'))


### Main Program ###
ACTIONS = {
    'check': check,
    'run': run,
}


def get_args():
    parser = ArgumentParser(description='Mock shell action')
    parser.add_argument(
        '--uid',
        default='1000',
        help='User ID to build the correct /var/run/user path',
    )
    subparsers = parser.add_subparsers(dest='action', required=True)

    # Check command.
    check = subparsers.add_parser('check')

    # Run command.
    run = subparsers.add_parser('run')
    run.add_argument('command', nargs=argparse.REMAINDER)

    # Parse and return.
    return parser.parse_args()


def main():
    args = get_args()
    action = ACTIONS[args.action]
    return action(args)


if __name__ == '__main__':
    main()
