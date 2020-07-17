# Systemd External Actions
Systemd helper scripts and example unit.

For details on systemd extertnal actions see
https://replicante.io/docs/agent/main/info/external-actions/#managing-actions-with-systemd


## Local example
```bash
# "deploy" action and systemd unit.
$ ln -s $PWD/tools/systemd/example.service ~/.config/systemd/user/example@.service
$ ln -s $PWD/tools/systemd/example-action /tmp/example-action
$ systemctl --user daemon-reload

# Edit agent config file to define a new action (see docs).
# Run agent and trigger the action (see docs).
```
