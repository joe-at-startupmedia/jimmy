# Jimmy

Jimmy is a language-agnostic, Redis-backed job queue server with an HTTP interface and a focus on long running tasks.
It was forked from Ocypod with additonal concerns in mind.

## Features

* simple setup - only requirement is Redis
* language agnostic - uses HTTP/JSON protocol, clients/workers can be
  implemented in any language
* long running jobs - handle jobs that may be running for hours/days,
  detect failure early using heartbeats
* simple HTTP interface - no complex binary protocols or client/worker logic
* flexible job metadata - allows for different patterns of use (e.g. progress
  tracking, partial results, etc.)
* job inspection - check the status of any jobs submitted to the system
* tagging - custom tags allow easy grouping and searching of related jobs
* automatic retries - re-queue jobs on failure or timeout
* Ability to retry jobs
