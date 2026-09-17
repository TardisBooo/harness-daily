name: Bug report
description: Something is broken
labels: ["bug"]
body:
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: Also tell us what you expected to happen.
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: harness-daily version
      placeholder: "harness-daily --version"
    validations:
      required: true
  - type: dropdown
    id: os
    attributes:
      label: OS
      options:
        - Windows
        - macOS
        - Linux
    validations:
      required: true
  - type: textarea
    id: doctor
    attributes:
      label: Output of `harness-daily doctor`
      description: Redact any personal paths if needed.
      render: text
  - type: textarea
    id: logs
    attributes:
      label: Relevant logs
      description: From `report --dry-collect` or the failed run. Redact secrets.
      render: text
