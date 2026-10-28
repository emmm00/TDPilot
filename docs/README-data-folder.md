# TDPilot - data folder

Everything below is generated at runtime. Nothing here is part of the
repository; this file only exists so the folder has something to delete.

    config.json     UI language, refresh interval, last applied value
    ApplyLimit.cmd  helper that the scheduled task runs (plain ASCII batch)
    limit-mw.txt    the power cap in milliwatts, read by ApplyLimit.cmd
    apply.log       one line per automatic re-apply, including read-back result
    tdpilot.log     optional app log

To reset the app completely: close it, delete this folder, and remove the
`TDPilotApply` scheduled task (the Automation page has a Remove button).
