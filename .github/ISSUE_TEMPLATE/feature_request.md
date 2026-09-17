name: Feature request
description: New harness, writer backend, or report improvement
labels: ["enhancement"]
body:
  - type: textarea
    id: problem
    attributes:
      label: What problem does this solve?
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposed solution
      description: For a new harness, describe where its sessions live and their JSONL shape (synthetic example).
    validations:
      required: true
